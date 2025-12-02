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
    
    /// 检查是否有公钥
    pub fn has_public_key(&self) -> bool {
        self.public_key.is_some()
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
        
        // 检查是否包含公钥标记（防止公钥被当作license code）
        if license_code.contains("BEGIN") || license_code.contains("PUBLIC KEY") {
            return Err(AppError::Auth("许可证格式无效，请确保输入的是正确的许可证代码，而不是公钥".to_string()));
        }
        
        let public_key = self.public_key.as_ref()
            .ok_or_else(|| AppError::Auth("公钥未加载，请先获取公钥".to_string()))?;
        
        // Base64解码
        let encrypted_data = general_purpose::STANDARD.decode(license_code)
            .map_err(|e| AppError::Auth(format!("License解码失败: {}", e)))?;
        
        // 使用RSA公钥解密私钥加密的数据
        // 在RSA中，如果数据是用私钥加密的（使用PKCS1v15填充），可以用公钥解密
        // 从toolset的代码看，使用的是PKCS1_PADDING，对应Rust中的Pkcs1v15
        
        // 在rsa crate中，我们需要使用公钥的原始RSA操作来解密
        // 由于rsa crate的API限制，我们需要访问公钥的内部结构
        
        // 先尝试直接解析UTF-8（用于未加密的数据，开发/测试环境）
        match String::from_utf8(encrypted_data.clone()) {
            Ok(decrypted_str) => {
                // 数据没有被加密，直接解析JSON
                let license_data: LicenseData = serde_json::from_str(&decrypted_str)
                    .map_err(|e| AppError::Auth(format!("License数据解析失败: {}", e)))?;
                Ok(license_data)
            }
            Err(_) => {
                // 数据是加密的，需要使用公钥解密
                // 使用rsa crate的底层API进行解密
                use rsa::traits::PublicKeyParts;
                
                // 获取公钥的模数和指数
                let n = public_key.n();
                let e = public_key.e();
                
                // 使用RSA原始操作：decrypted = encrypted^e mod n
                // 将加密数据转换为BigUint
                use rsa::BigUint;
                
                let encrypted_bigint = BigUint::from_bytes_be(&encrypted_data);
                
                // 计算：decrypted = encrypted^e mod n
                let decrypted_bigint = encrypted_bigint.modpow(e, n);
                
                // 将解密后的数据转换回字节
                // 注意：RSA 密钥大小通常是 2048 位（256 字节），所以解密后的数据应该是 256 字节
                let mut decrypted_bytes = decrypted_bigint.to_bytes_be();
                
                // 确保数据长度正确（可能需要前导零）
                let key_size = (public_key.n().bits() + 7) / 8; // 密钥大小（字节）
                if decrypted_bytes.len() < key_size {
                    // 在前面补零
                    let mut padded = vec![0u8; key_size - decrypted_bytes.len()];
                    padded.extend_from_slice(&decrypted_bytes);
                    decrypted_bytes = padded;
                }
                
                // 移除PKCS1v15填充
                // PKCS1v15加密格式：00 02 [随机填充（非零字节）] 00 [数据]
                // 我们需要找到数据开始的位置（跳过填充）
                let mut data_start = None;
                
                // 检查格式：必须以 00 02 开头
                if decrypted_bytes.len() >= 2 && decrypted_bytes[0] == 0x00 && decrypted_bytes[1] == 0x02 {
                    // 从位置 2 开始查找 00 标记（填充结束）
                    for i in 2..decrypted_bytes.len() {
                        if decrypted_bytes[i] == 0x00 {
                            // 找到填充结束标记，下一个字节开始是数据
                            if i + 1 < decrypted_bytes.len() {
                                data_start = Some(i + 1);
                                break;
                            }
                        }
                    }
                }
                
                // 提取实际数据
                let actual_data = if let Some(start) = data_start {
                    &decrypted_bytes[start..]
                } else {
                    // 如果没有找到标准的 PKCS1v15 填充，尝试其他方法
                    // 可能数据格式不同，或者填充方式不同
                    // 尝试查找第一个可打印字符的位置
                    let mut found_start = None;
                    for (i, &byte) in decrypted_bytes.iter().enumerate() {
                        // 查找 JSON 开始字符 '{' 或 '[' 
                        if byte == b'{' || byte == b'[' {
                            found_start = Some(i);
                            break;
                        }
                    }
                    
                    if let Some(start) = found_start {
                        &decrypted_bytes[start..]
                    } else {
                        // 最后尝试：跳过前导零
                        let mut start = 0;
                        for (i, &byte) in decrypted_bytes.iter().enumerate() {
                            if byte != 0x00 {
                                start = i;
                                break;
                            }
                        }
                        &decrypted_bytes[start..]
                    }
                };
                
                // 尝试解析为UTF-8字符串
                let decrypted_str = String::from_utf8(actual_data.to_vec())
                    .map_err(|e| {
                        // 添加详细的调试信息
                        let hex_bytes: String = actual_data.iter()
                            .take(200) // 显示前200个字节
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        
                        // 尝试以可打印字符形式显示（替换不可打印字符）
                        let printable: String = actual_data.iter()
                            .take(200)
                            .map(|&b| {
                                if b >= 32 && b <= 126 {
                                    b as char
                                } else {
                                    '.'
                                }
                            })
                            .collect();
                        
                        // 尝试使用 lossy 转换显示（即使不是有效的UTF-8）
                        let lossy_str = String::from_utf8_lossy(actual_data);
                        
                        log::error!("======== 解密数据调试信息 ========");
                        log::error!("解密后的数据长度: {} 字节", actual_data.len());
                        log::error!("前200字节（十六进制）: {}", hex_bytes);
                        log::error!("前200字节（可打印字符）: {}", printable);
                        log::error!("完整数据（lossy UTF-8）: {}", lossy_str);
                        log::error!("UTF-8错误详情: {}", e);
                        log::error!("====================================");
                        
                        AppError::Auth(format!(
                            "License解密后数据格式错误: {} (数据长度: {} 字节, 可打印预览: {})",
                            e,
                            actual_data.len(),
                            printable.chars().take(50).collect::<String>()
                        ))
                    })?;
                
                log::info!("解密成功，解密后的字符串: {}", decrypted_str);
                
                // 解析JSON
                let license_data: LicenseData = serde_json::from_str(&decrypted_str)
                    .map_err(|e| AppError::Auth(format!("License数据解析失败: {}", e)))?;
                
                Ok(license_data)
            }
        }
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

