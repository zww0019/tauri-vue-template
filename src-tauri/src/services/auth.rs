use crate::error::{AppError, AppResult};
use crate::services::storage::StorageService;
use crate::types::{AuthStatus, DeviceInfo, LicenseData, ToolLicense};
use base64::{engine::general_purpose, Engine as _};
use rsa::RsaPublicKey;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// 认证服务 - 负责License验证和管理
pub struct AuthService {
    storage: Arc<StorageService>,
    server_url: String,
    public_key: Option<RsaPublicKey>,
}

impl AuthService {
    pub fn new(storage: Arc<StorageService>) -> Self {
        let server_url = std::env::var("LICENSE_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let mut service = Self {
            storage,
            server_url,
            public_key: None,
        };
        
        // 尝试从本地存储加载公钥
        service.load_cached_public_key();
        
        service
    }
    
    /// 从本地缓存加载公钥
    fn load_cached_public_key(&mut self) {
        if let Some(pem_str) = self.storage.get::<String>("license_public_key") {
            match self.parse_public_key(&pem_str) {
                Ok(key) => {
                    self.public_key = Some(key);
                    log::info!("✓ 从本地缓存加载公钥成功");
                }
                Err(e) => {
                    log::error!("解析缓存的公钥失败: {}", e);
                }
            }
        }
    }
    
    /// 解析PEM格式的公钥
    fn parse_public_key(&self, pem_str: &str) -> AppResult<RsaPublicKey> {
        use rsa::pkcs8::DecodePublicKey;
        
        RsaPublicKey::from_public_key_pem(pem_str)
            .map_err(|e| AppError::Auth(format!("解析公钥失败: {}", e)))
    }
    
    /// 从服务器获取公钥
    pub async fn fetch_public_key(&mut self) -> AppResult<String> {
        let url = format!("{}/api/public-key", self.server_url);
        
        let response = reqwest::get(&url)
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let public_key_str = response["publicKey"]
            .as_str()
            .ok_or_else(|| AppError::Auth("服务器响应中未包含公钥".to_string()))?
            .to_string();
        
        // 验证并保存公钥
        let public_key = self.parse_public_key(&public_key_str)?;
        self.public_key = Some(public_key);
        
        // 缓存公钥到本地
        self.storage.set("license_public_key", &public_key_str)?;
        
        log::info!("✓ 从服务器获取公钥成功");
        Ok(public_key_str)
    }
    
    /// 验证license code（本地验证）
    fn verify_license_code(&self, license_code: &str) -> AppResult<LicenseData> {
        // 验证格式
        if license_code.len() < 50 {
            return Err(AppError::Auth("License格式无效".to_string()));
        }
        
        let _public_key = self.public_key.as_ref()
            .ok_or_else(|| AppError::Auth("公钥未加载，请先获取公钥".to_string()))?;
        
        // Base64解码
        let encrypted_data = general_purpose::STANDARD.decode(license_code)
            .map_err(|e| AppError::Auth(format!("License解码失败: {}", e)))?;
        
        // 注意：RSA公钥无法解密，这里需要重新设计验证逻辑
        // 临时方案：直接解析base64后的JSON（实际应该用签名验证）
        let decrypted_str = String::from_utf8(encrypted_data)
            .map_err(|e| AppError::Auth(format!("License数据格式错误: {}", e)))?;
        
        // 解析JSON
        let license_data: LicenseData = serde_json::from_str(&decrypted_str)
            .map_err(|e| AppError::Auth(format!("License数据解析失败: {}", e)))?;
        
        Ok(license_data)
    }
    
    /// 获取设备指纹
    fn get_device_fingerprint(&self) -> String {
        let device_info = format!(
            "{}:{}:{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            self.storage.get_data_dir().to_string_lossy()
        );
        
        let mut hasher = Sha256::new();
        hasher.update(device_info.as_bytes());
        let result = hasher.finalize();
        
        format!("{:x}", result)
    }
    
    /// 获取设备信息
    fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            fingerprint: self.get_device_fingerprint(),
            platform: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// 注册工具License
    pub async fn register_tool_license(
        &self,
        tool_id: &str,
        license_code: &str,
    ) -> AppResult<ApiResponse<()>> {
        // 验证License格式
        if license_code.len() < 50 {
            return Ok(ApiResponse::error("许可证格式无效"));
        }
        
        // 本地验证License
        let license_data = self.verify_license_code(license_code)?;
        
        // 验证工具ID是否匹配
        if license_data.tool_id != tool_id {
            return Ok(ApiResponse::error("许可证与工具ID不匹配"));
        }
        
        // 检查是否已注册
        let tool_licenses: HashMap<String, ToolLicense> = self
            .storage
            .get("tool_licenses")
            .unwrap_or_default();
        
        if tool_licenses.contains_key(tool_id) {
            return Ok(ApiResponse::error("该工具已注册，请先解绑后再注册新授权码"));
        }
        
        // 向服务器验证并绑定设备
        let device_info = self.get_device_info();
        let url = format!("{}/api/verify", self.server_url);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&serde_json::json!({
                "licenseCode": license_code,
                "toolId": tool_id,
                "deviceInfo": device_info
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        if !response["success"].as_bool().unwrap_or(false) {
            let message = response["message"]
                .as_str()
                .unwrap_or("服务器验证失败");
            return Ok(ApiResponse::error(message));
        }
        
        // 保存License信息
        let mut tool_licenses = tool_licenses;
        tool_licenses.insert(
            tool_id.to_string(),
            ToolLicense {
                license_code: license_code.to_string(),
                license_data: license_data.clone(),
                device_fingerprint: device_info.fingerprint,
                registered_date: chrono::Utc::now().to_rfc3339(),
                server_verified: true,
                last_verified: Some(chrono::Utc::now().to_rfc3339()),
            },
        );
        
        self.storage.set("tool_licenses", &tool_licenses)?;
        
        Ok(ApiResponse::success_with_message("注册成功", ()))
    }
    
    /// 检查工具认证状态
    pub async fn check_tool_auth_status(&self, tool_id: &str) -> AppResult<AuthStatus> {
        let tool_licenses: HashMap<String, ToolLicense> = self
            .storage
            .get("tool_licenses")
            .unwrap_or_default();
        
        let tool_license = match tool_licenses.get(tool_id) {
            Some(license) => license,
            None => {
                return Ok(AuthStatus {
                    is_authenticated: false,
                    license_data: None,
                });
            }
        };
        
        // 本地验证License
        let license_data = match self.verify_license_code(&tool_license.license_code) {
            Ok(data) => data,
            Err(_) => {
                // License无效，清除存储
                let mut tool_licenses = tool_licenses;
                tool_licenses.remove(tool_id);
                self.storage.set("tool_licenses", &tool_licenses)?;
                
                return Ok(AuthStatus {
                    is_authenticated: false,
                    license_data: None,
                });
            }
        };
        
        // 验证工具ID
        if license_data.tool_id != tool_id {
            let mut tool_licenses = tool_licenses;
            tool_licenses.remove(tool_id);
            self.storage.set("tool_licenses", &tool_licenses)?;
            
            return Ok(AuthStatus {
                is_authenticated: false,
                license_data: None,
            });
        }
        
        // 验证过期时间
        let expiry_date = chrono::DateTime::parse_from_rfc3339(&license_data.expiry_date)
            .map_err(|e| AppError::Auth(format!("解析过期时间失败: {}", e)))?;
        
        if expiry_date < chrono::Utc::now() {
            let mut tool_licenses = tool_licenses;
            tool_licenses.remove(tool_id);
            self.storage.set("tool_licenses", &tool_licenses)?;
            
            return Ok(AuthStatus {
                is_authenticated: false,
                license_data: None,
            });
        }
        
        // 尝试向服务器验证（可选，失败不影响本地验证结果）
        // 这里简化处理，实际可以添加服务器验证逻辑
        
        Ok(AuthStatus {
            is_authenticated: true,
            license_data: Some(license_data),
        })
    }
    
    /// 获取所有已注册的工具
    pub async fn get_registered_tools(&self) -> AppResult<Vec<String>> {
        let tool_licenses: HashMap<String, ToolLicense> = self
            .storage
            .get("tool_licenses")
            .unwrap_or_default();
        
        let mut registered_tools = Vec::new();
        
        for (tool_id, _) in tool_licenses.iter() {
            let auth_status = self.check_tool_auth_status(tool_id).await?;
            if auth_status.is_authenticated {
                registered_tools.push(tool_id.clone());
            }
        }
        
        Ok(registered_tools)
    }
    
    /// 重置工具认证
    pub fn reset_tool_auth(&self, tool_id: Option<&str>) -> AppResult<()> {
        if let Some(tool_id) = tool_id {
            let mut tool_licenses: HashMap<String, ToolLicense> = self
                .storage
                .get("tool_licenses")
                .unwrap_or_default();
            
            tool_licenses.remove(tool_id);
            self.storage.set("tool_licenses", &tool_licenses)?;
        } else {
            self.storage.remove("tool_licenses")?;
        }
        
        Ok(())
    }
}

use crate::error::ApiResponse;

