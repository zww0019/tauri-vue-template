use crate::error::{AppError, AppResult};
use crate::types::{ToolUpdateInfo, ToolVersionConfig};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;

/// 工具更新服务 - 负责从远程服务器检查和下载工具
pub struct ToolUpdateService {
    update_url: String,
    tools_dir: PathBuf,
    download_dir: PathBuf,
    current_versions: HashMap<String, String>,
}

impl ToolUpdateService {
    pub fn new(app_handle: &tauri::AppHandle) -> AppResult<Self> {
        let update_url = std::env::var("TOOLS_UPDATE_URL")
            .unwrap_or_else(|_| "http://toolset.zwwpc.top/toolset/tools/tools-version.json".to_string());
        
        // 获取工具目录（与 ToolsManager 保持一致）
        let tools_dir = if cfg!(debug_assertions) {
            let user_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Tool(format!("无法获取用户数据目录: {}", e)))?;
            let user_data_tools = user_data_dir.join("tools");
            
            if user_data_tools.exists() {
                user_data_tools
            } else {
                let resource_path = app_handle
                    .path()
                    .resource_dir()
                    .map_err(|e| AppError::Tool(format!("无法获取资源目录: {}", e)))?;
                resource_path.join("tools")
            }
        } else {
            let user_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Tool(format!("无法获取用户数据目录: {}", e)))?;
            user_data_dir.join("tools")
        };
        
        let download_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Tool(format!("无法获取用户数据目录: {}", e)))?
            .join("tool-downloads");
        
        // 确保下载目录存在
        if !download_dir.exists() {
            fs::create_dir_all(&download_dir)?;
        }
        
        let mut service = Self {
            update_url,
            tools_dir: tools_dir.clone(),
            download_dir,
            current_versions: HashMap::new(),
        };
        
        // 加载当前已安装的工具版本
        service.load_current_versions()?;
        
        Ok(service)
    }
    
    /// 加载当前已安装的工具版本
    fn load_current_versions(&mut self) -> AppResult<()> {
        log::info!("======== 加载当前工具版本 ========");
        log::info!("工具目录: {:?}", self.tools_dir);
        
        if !self.tools_dir.exists() {
            log::warn!("工具目录不存在，创建目录: {:?}", self.tools_dir);
            fs::create_dir_all(&self.tools_dir)?;
            log::info!("首次启动，工具目录为空");
            return Ok(());
        }
        
        let entries = fs::read_dir(&self.tools_dir)?;
        let mut count = 0;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let tool_id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());
                
                if let Some(tool_id) = tool_id {
                    let manifest_path = path.join("manifest.json");
                    if manifest_path.exists() {
                        match fs::read_to_string(&manifest_path) {
                            Ok(content) => {
                                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                                    if let Some(version) = manifest.get("version").and_then(|v| v.as_str()) {
                                        self.current_versions.insert(tool_id.clone(), version.to_string());
                                        log::info!("  {}: {}", tool_id, version);
                                        count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("  {}: 读取 manifest.json 失败 - {}", tool_id, e);
                            }
                        }
                    } else {
                        log::info!("  {}: 缺少 manifest.json", tool_id);
                    }
                }
            }
        }
        
        log::info!("======== 工具版本加载完成 ========");
        if count > 0 {
            let tool_names: Vec<String> = self.current_versions.keys().cloned().collect();
            log::info!("已安装的工具: {}", tool_names.join(", "));
        } else {
            log::info!("当前无已安装的工具");
        }
        log::info!("=============================");
        
        Ok(())
    }
    
    /// 检查工具更新
    pub async fn check_for_updates(&self) -> AppResult<Vec<ToolUpdateInfo>> {
        log::info!("======== 开始检查工具更新 ========");
        log::info!("更新源地址: {}", self.update_url);
        
        let installed_count = self.current_versions.len();
        log::info!("当前已安装 {} 个工具: {:?}", installed_count, self.current_versions.keys().collect::<Vec<_>>());
        
        let remote_config = self.fetch_remote_versions().await?;
        let remote_tools = remote_config.tools.keys().count();
        log::info!("远程可用 {} 个工具: {:?}", remote_tools, remote_config.tools.keys().collect::<Vec<_>>());
        
        let mut updates = Vec::new();
        let mut new_tools = Vec::new();
        
        for (tool_id, remote_info) in remote_config.tools {
            let current_version = self.current_versions.get(&tool_id).cloned().unwrap_or_else(|| "0.0.0".to_string());
            let remote_version = remote_info.version.clone();
            let is_new_tool = current_version == "0.0.0";
            
            log::info!("比较 {}: 当前 {} vs 远程 {}", tool_id, current_version, remote_version);
            
            if Self::compare_versions(&remote_version, &current_version) > 0 {
                let update_info = ToolUpdateInfo {
                    tool_id: tool_id.clone(),
                    current_version,
                    remote_version: remote_version.clone(),
                    download_url: remote_info.download_url.clone(),
                    checksum: remote_info.checksum.clone(),
                    size: remote_info.size,
                    release_notes: remote_info.release_notes.clone(),
                };
                
                if is_new_tool {
                    log::info!("  → 发现新工具: {} v{}", tool_id, remote_version);
                    new_tools.push(tool_id.clone());
                } else {
                    log::info!("  → 发现更新: {} ({} → {})", tool_id, update_info.current_version, remote_version);
                }
                updates.push(update_info);
            } else {
                log::info!("  → 已是最新版本: {}", tool_id);
            }
        }
        
        log::info!("======== 检查完成 ========");
        log::info!("发现 {} 个可用更新 (其中 {} 个新工具)", updates.len(), new_tools.len());
        if !new_tools.is_empty() {
            log::info!("新工具: {}", new_tools.join(", "));
        }
        log::info!("========================");
        
        Ok(updates)
    }
    
    /// 获取远程版本信息
    async fn fetch_remote_versions(&self) -> AppResult<ToolVersionConfig> {
        log::info!("开始获取远程版本信息: {}", self.update_url);
        
        let response = reqwest::get(&self.update_url).await
            .map_err(|e| AppError::Tool(format!("网络请求失败: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Tool(format!("HTTP {}", response.status())));
        }
        
        let content = response.text().await
            .map_err(|e| AppError::Tool(format!("读取响应失败: {}", e)))?;
        
        log::info!("接收到的数据长度: {}", content.len());
        
        let config: ToolVersionConfig = serde_json::from_str(&content)
            .map_err(|e| AppError::Tool(format!("解析远程版本信息失败: {}", e)))?;
        
        log::info!("成功解析远程版本信息");
        Ok(config)
    }
    
    /// 下载工具包
    pub async fn download_tool(
        &self,
        tool_id: &str,
        download_url: &str,
        on_progress: Option<Box<dyn Fn(u64, u64, f64) + Send + Sync>>,
    ) -> AppResult<PathBuf> {
        log::info!("开始下载工具: {}", tool_id);
        log::info!("下载地址: {}", download_url);
        
        let file_name = format!("{}-{}.zip", tool_id, chrono::Utc::now().timestamp());
        let file_path = self.download_dir.join(&file_name);
        
        log::info!("保存路径: {:?}", file_path);
        
        let mut response = reqwest::get(download_url).await
            .map_err(|e| AppError::Tool(format!("下载失败: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Tool(format!("HTTP {}", response.status())));
        }
        
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded_size = 0u64;
        let mut file = fs::File::create(&file_path)
            .map_err(|e| AppError::Tool(format!("创建文件失败: {}", e)))?;
        
        while let Some(chunk) = response.chunk().await
            .map_err(|e| AppError::Tool(format!("读取数据失败: {}", e)))? {
            file.write_all(&chunk)
                .map_err(|e| AppError::Tool(format!("写入文件失败: {}", e)))?;
            downloaded_size += chunk.len() as u64;
            
            if let Some(ref callback) = on_progress {
                let progress = if total_size > 0 {
                    (downloaded_size as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                };
                callback(downloaded_size, total_size, progress);
            }
        }
        
        log::info!("工具下载完成: {}", tool_id);
        Ok(file_path)
    }
    
    /// 安装工具包
    pub async fn install_tool(&mut self, tool_id: &str, zip_path: &PathBuf, checksum: &str) -> AppResult<()> {
        log::info!("开始安装工具: {}", tool_id);
        
        // 验证校验和（如果提供）
        if !checksum.is_empty() {
            let file_checksum = self.calculate_checksum(zip_path).await?;
            if file_checksum != checksum {
                return Err(AppError::Tool("文件校验失败".to_string()));
            }
        }
        
        // 解压工具包
        let tool_path = self.tools_dir.join(tool_id);
        
        // 备份旧版本
        if tool_path.exists() {
            let backup_path = format!("{}.backup", tool_path.display());
            if PathBuf::from(&backup_path).exists() {
                fs::remove_dir_all(&backup_path)?;
            }
            fs::rename(&tool_path, &backup_path)
                .map_err(|e| AppError::Tool(format!("备份失败: {}", e)))?;
        }
        
        // 解压新版本
        self.unzip(zip_path, &tool_path).await?;
        
        // 删除下载文件
        fs::remove_file(zip_path)
            .map_err(|e| AppError::Tool(format!("删除下载文件失败: {}", e)))?;
        
        // 更新版本信息
        let manifest_path = tool_path.join("manifest.json");
        if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path)?;
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                if let Some(version) = manifest.get("version").and_then(|v| v.as_str()) {
                    self.current_versions.insert(tool_id.to_string(), version.to_string());
                }
            }
        }
        
        log::info!("工具安装完成: {}", tool_id);
        Ok(())
    }
    
    /// 解压文件
    async fn unzip(&self, zip_path: &PathBuf, dest_path: &PathBuf) -> AppResult<()> {
        
        let file = fs::File::open(zip_path)
            .map_err(|e| AppError::Tool(format!("打开压缩文件失败: {}", e)))?;
        
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Tool(format!("读取压缩文件失败: {}", e)))?;
        
        if !dest_path.exists() {
            fs::create_dir_all(dest_path)?;
        }
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| AppError::Tool(format!("读取压缩文件条目失败: {}", e)))?;
            
            let outpath = match file.enclosed_name() {
                Some(path) => dest_path.join(path),
                None => continue,
            };
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| AppError::Tool(format!("创建文件失败: {}", e)))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| AppError::Tool(format!("解压文件失败: {}", e)))?;
            }
        }
        
        log::info!("解压完成: {:?} -> {:?}", zip_path, dest_path);
        Ok(())
    }
    
    /// 计算文件校验和
    async fn calculate_checksum(&self, file_path: &PathBuf) -> AppResult<String> {
        use sha2::{Sha256, Digest};
        use std::io::Read;
        
        let mut file = fs::File::open(file_path)
            .map_err(|e| AppError::Tool(format!("打开文件失败: {}", e)))?;
        
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| AppError::Tool(format!("读取文件失败: {}", e)))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// 比较版本号
    fn compare_versions(v1: &str, v2: &str) -> i32 {
        let parts1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
        let parts2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();
        
        let max_len = parts1.len().max(parts2.len());
        
        for i in 0..max_len {
            let p1 = parts1.get(i).copied().unwrap_or(0);
            let p2 = parts2.get(i).copied().unwrap_or(0);
            
            if p1 > p2 {
                return 1;
            } else if p1 < p2 {
                return -1;
            }
        }
        
        0
    }
    
    /// 清理备份
    pub fn cleanup_backups(&self) -> AppResult<()> {
        if !self.tools_dir.exists() {
            return Ok(());
        }
        
        let entries = fs::read_dir(&self.tools_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".backup") {
                    fs::remove_dir_all(&path)?;
                    log::info!("已清理备份: {}", name);
                }
            }
        }
        Ok(())
    }
    
    /// 获取工具目录
    pub fn get_tools_dir(&self) -> &PathBuf {
        &self.tools_dir
    }
}

