use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};
use once_cell::sync::Lazy;

// 公钥管理器状态
pub struct PublicKeyManager {
    public_key: Arc<Mutex<Option<String>>>,
    is_retrying: Arc<AtomicBool>,
    retry_count: Arc<AtomicU32>,
    server_url: String,
    max_retry_delay: u64, // 最大重试延迟（毫秒）
}

impl PublicKeyManager {
    pub fn new(server_url: String) -> Self {
        Self {
            public_key: Arc::new(Mutex::new(None)),
            is_retrying: Arc::new(AtomicBool::new(false)),
            retry_count: Arc::new(AtomicU32::new(0)),
            server_url,
            max_retry_delay: 64000, // 64秒
        }
    }

    // 获取公钥文件路径
    fn get_public_key_path(app: &AppHandle) -> PathBuf {
        let app_data_dir = app.path().app_data_dir().unwrap();
        std::fs::create_dir_all(&app_data_dir).unwrap();
        app_data_dir.join("license_public_key")
    }

    // 从本地加载公钥
    pub async fn load_public_key_from_local(app: &AppHandle) -> Option<String> {
        let key_path = Self::get_public_key_path(app);
        if key_path.exists() {
            if let Ok(key) = fs::read_to_string(&key_path) {
                if Self::validate_public_key(&key) {
                    return Some(key);
                }
            }
        }
        None
    }

    // 保存公钥到本地
    async fn save_public_key_to_local(app: &AppHandle, public_key: &str) -> Result<(), String> {
        let key_path = Self::get_public_key_path(app);
        fs::write(&key_path, public_key)
            .map_err(|e| format!("保存公钥失败: {}", e))?;
        Ok(())
    }

    // 验证公钥格式
    fn validate_public_key(public_key: &str) -> bool {
        if public_key.is_empty() || public_key.len() < 100 {
            return false;
        }
        // 公钥应该是PEM格式，包含 BEGIN 和 END 标记
        public_key.contains("BEGIN") && public_key.contains("PUBLIC KEY")
    }

    // 计算重试延迟（指数退避策略）
    fn calculate_retry_delay(&self, retry_count: u32) -> u64 {
        let base_delay = 1000u64; // 基础延迟：1秒
        let exponent = (retry_count - 1).min(6);
        let exponential_delay = (2u64.pow(exponent)) * base_delay;
        exponential_delay.min(self.max_retry_delay)
    }

    // 从服务器获取公钥（单次尝试）
    async fn fetch_public_key(&self) -> Result<String, String> {
        let url = format!("{}/api/public-key", self.server_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("网络请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("服务器返回错误: {}", response.status()));
        }

        // 定义响应结构
        #[derive(Deserialize)]
        struct PublicKeyResponse {
            success: bool,
            #[serde(rename = "publicKey")]
            public_key: Option<String>,
        }

        // 解析 JSON 响应
        let data: PublicKeyResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        if !data.success {
            return Err("服务器响应中 success 为 false".to_string());
        }

        let public_key = data.public_key
            .ok_or_else(|| "服务器响应中未包含 publicKey 字段".to_string())?;

        // 验证公钥格式
        if !Self::validate_public_key(&public_key) {
            return Err("从服务器获取的公钥格式无效".to_string());
        }

        Ok(public_key)
    }

    // 带重试机制的公钥获取
    async fn fetch_public_key_with_retry(&self, app: AppHandle) {
        // 如果已经在重试中，不要重复启动
        if self.is_retrying.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return;
        }

        // 先尝试从本地加载
        if let Some(local_key) = Self::load_public_key_from_local(&app).await {
            let mut key_guard = self.public_key.lock().await;
            *key_guard = Some(local_key.clone());
            drop(key_guard);
            println!("已有本地公钥缓存，尝试获取最新版本...");
        } else {
            println!("未找到本地公钥缓存，开始获取公钥...");
        }

        loop {
            match self.fetch_public_key().await {
                Ok(new_key) => {
                    // 检查是否与现有公钥不同
                    let mut key_guard = self.public_key.lock().await;
                    let is_new_key = key_guard.as_ref().map(|k| k != &new_key).unwrap_or(true);
                    
                    *key_guard = Some(new_key.clone());
                    drop(key_guard);

                    // 保存到本地
                    if let Err(e) = Self::save_public_key_to_local(&app, &new_key).await {
                        eprintln!("保存公钥到本地失败: {}", e);
                    }

                    if is_new_key {
                        println!("✓ 公钥已更新并保存到本地");
                    } else {
                        println!("✓ 公钥获取成功（与本地一致）");
                    }

                    let retry_count = self.retry_count.load(Ordering::Relaxed);
                    if retry_count > 0 {
                        println!("✓ 公钥获取成功 (尝试次数: {})", retry_count + 1);
                    }

                    // 重置计数器
                    self.retry_count.store(0, Ordering::Release);
                    self.is_retrying.store(false, Ordering::Release);
                    return;
                }
                Err(e) => {
                    let retry_count = self.retry_count.fetch_add(1, Ordering::Relaxed) + 1;
                    let delay = self.calculate_retry_delay(retry_count);
                    
                    eprintln!("× 公钥获取失败 (第 {} 次尝试): {}", retry_count, e);
                    println!("{}秒后重试...", delay / 1000);

                    // 等待后重试
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    // 启动自动重试获取公钥
    pub fn start_auto_retry_fetch_public_key(&self, app: AppHandle) {
        let manager = Arc::new(self.clone());
        let app_clone = app.clone();
        
        // 使用 Tauri 的异步运行时
        tauri::async_runtime::spawn(async move {
            manager.fetch_public_key_with_retry(app_clone).await;
        });
    }

    // 手动触发重新获取公钥
    pub async fn refresh_public_key(&self, app: AppHandle) {
        println!("手动触发公钥刷新...");
        
        // 重置状态
        self.is_retrying.store(false, Ordering::Release);
        self.retry_count.store(0, Ordering::Release);
        
        // 重新启动获取流程
        self.start_auto_retry_fetch_public_key(app);
    }

    // 获取当前公钥
    pub async fn get_public_key(&self) -> Option<String> {
        self.public_key.lock().await.clone()
    }
}

// 实现 Clone for PublicKeyManager
impl Clone for PublicKeyManager {
    fn clone(&self) -> Self {
        Self {
            public_key: Arc::clone(&self.public_key),
            is_retrying: Arc::clone(&self.is_retrying),
            retry_count: Arc::clone(&self.retry_count),
            server_url: self.server_url.clone(),
            max_retry_delay: self.max_retry_delay,
        }
    }
}

// 全局公钥管理器
pub static PUBLIC_KEY_MANAGER: Lazy<Arc<Mutex<Option<PublicKeyManager>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

// 初始化公钥管理器
pub async fn init_public_key_manager(server_url: String, app: AppHandle) {
    let manager = PublicKeyManager::new(server_url);
    
    // 先尝试从本地加载
    if let Some(local_key) = PublicKeyManager::load_public_key_from_local(&app).await {
        let mut key_guard = manager.public_key.lock().await;
        *key_guard = Some(local_key);
    }
    
    // 启动自动获取
    manager.start_auto_retry_fetch_public_key(app.clone());
    
    // 保存到全局
    let mut global_manager = PUBLIC_KEY_MANAGER.lock().await;
    *global_manager = Some(manager);
}

// 手动刷新公钥
#[tauri::command]
pub async fn refresh_public_key(app: AppHandle) -> Result<(), String> {
    let manager_guard = PUBLIC_KEY_MANAGER.lock().await;
    if let Some(ref manager) = *manager_guard {
        manager.refresh_public_key(app).await;
        Ok(())
    } else {
        Err("公钥管理器未初始化".to_string())
    }
}

// 获取当前公钥
#[tauri::command]
pub async fn get_public_key() -> Result<Option<String>, String> {
    let manager_guard = PUBLIC_KEY_MANAGER.lock().await;
    if let Some(ref manager) = *manager_guard {
        Ok(manager.get_public_key().await)
    } else {
        Ok(None)
    }
}

