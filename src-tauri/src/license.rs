use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use rsa::{RsaPublicKey, BigUint};
use rsa::pkcs1::DecodeRsaPublicKey;
use pkcs8::DecodePublicKey;
use base64::{Engine as _, engine::general_purpose};
use crate::public_key::{PUBLIC_KEY_MANAGER, PublicKeyManager};

// 授权码明文格式
// JSON 字段使用 camelCase，Rust 字段使用 snake_case
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseInfo {
    pub license_id: String,
    pub tool_id: String,
    pub licensee_id: String,
    pub issued_date: String,
    pub expiry_date: String,
    pub max_devices: u32,
    pub features: Vec<String>,
    pub version: String,
}

// 获取授权文件路径
fn get_license_file_path(app: &AppHandle) -> PathBuf {
    let app_data_dir = app.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_data_dir).unwrap();
    app_data_dir.join("license.enc")
}

// 保存授权信息（加密）
fn save_license(app: &AppHandle, encrypted_license: &str) -> Result<(), String> {
    use base64::{Engine as _, engine::general_purpose};
    
    let license_path = get_license_file_path(app);
    
    // 使用简单的加密（实际应该使用 AES 加密）
    let encrypted = general_purpose::STANDARD.encode(encrypted_license.as_bytes());
    
    fs::write(&license_path, encrypted)
        .map_err(|e| format!("保存授权信息失败: {}", e))?;
    
    Ok(())
}

// RSA 公钥解密授权码
// 授权码格式：base64(RSA私钥加密的JSON数据)，使用 PKCS#1 v1.5 填充
fn decrypt_license_with_rsa(
    encrypted_license: &str,
    public_key_pem: &str,
) -> Result<String, String> {
    // 1. 解析 PEM 格式的公钥
    // 支持 PKCS#8 格式 (-----BEGIN PUBLIC KEY-----) 和 PKCS#1 格式 (-----BEGIN RSA PUBLIC KEY-----)
    let public_key = DecodePublicKey::from_public_key_pem(public_key_pem)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(public_key_pem))
        .map_err(|e| format!("解析公钥失败: {} (支持格式: PKCS#8 或 PKCS#1)", e))?;
    
    // 2. Base64 解码加密数据
    let encrypted_data = general_purpose::STANDARD
        .decode(encrypted_license)
        .map_err(|_| "授权码格式错误（Base64解码失败）".to_string())?;
    
    // 3. 使用 RSA 公钥解密（私钥加密的数据）
    // 授权码是用服务器私钥加密的，客户端用公钥解密
    // 这确保只有服务器能生成有效授权码，无法伪造
    let decrypted_data = decrypt_with_public_key(&public_key, &encrypted_data)?;
    
    // 4. 转换为字符串
    String::from_utf8(decrypted_data)
        .map_err(|_| "授权码解密后不是有效的UTF-8编码".to_string())
}

// 使用公钥解密（私钥加密的数据，使用 PKCS#1 v1.5 填充）
// 对应 Node.js 的 crypto.publicDecrypt with RSA_PKCS1_PADDING
fn decrypt_with_public_key(
    public_key: &RsaPublicKey,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, String> {
    use rsa::traits::PublicKeyParts;
    
    // RSA解密原理：
    // 如果数据是用私钥加密的：encrypted = data^d mod n (d是私钥指数)
    // 那么用公钥解密：data = encrypted^e mod n (e是公钥指数)
    
    // 1. 将加密数据转换为大整数（使用 rsa 内部的 BigUint）
    let encrypted_bigint = BigUint::from_bytes_be(encrypted_data);
    
    // 2. 获取公钥参数
    let n = public_key.n();
    let e = public_key.e();
    
    // 3. 执行模幂运算：decrypted = encrypted^e mod n
    let decrypted_bigint = encrypted_bigint.modpow(e, n);
    
    // 4. 转换回字节数组（需要填充到密钥长度）
    let key_size = (n.bits() + 7) / 8; // 密钥长度（字节）
    let mut decrypted_bytes = decrypted_bigint.to_bytes_be();
    
    // 确保长度正确（可能需要前导零）
    while decrypted_bytes.len() < key_size {
        decrypted_bytes.insert(0, 0);
    }
    
    // 5. 移除 PKCS#1 v1.5 填充
    // PKCS#1 v1.5 填充格式（私钥加密/签名）：
    // EM = 0x00 || 0x01 || PS || 0x00 || M
    // 其中：
    // - 0x00: 前导零字节
    // - 0x01: 块类型（私钥加密使用 0x01）
    // - PS: 填充字节（全部为 0xFF）
    // - 0x00: 分隔符
    // - M: 原始消息
    
    // 检查最小长度
    if decrypted_bytes.len() < 11 {
        return Err("PKCS#1填充格式错误：数据太短".to_string());
    }
    
    // 检查第一个字节应该是 0x00
    if decrypted_bytes[0] != 0x00 {
        return Err("PKCS#1填充格式错误：第一个字节不是0x00".to_string());
    }
    
    // 检查第二个字节应该是 0x01（私钥加密的块类型）
    if decrypted_bytes[1] != 0x01 {
        return Err("PKCS#1填充格式错误：第二个字节不是0x01".to_string());
    }
    
    // 查找分隔符 0x00（跳过前两个字节后的第一个 0x00）
    let mut separator_pos = None;
    for i in 2..decrypted_bytes.len() {
        if decrypted_bytes[i] == 0x00 {
            separator_pos = Some(i);
            break;
        }
        // PKCS#1 v1.5 的填充字节应该是 0xFF
        if decrypted_bytes[i] != 0xFF {
            return Err("PKCS#1填充格式错误：填充字节不是0xFF".to_string());
        }
    }
    
    // 提取原始消息
    match separator_pos {
        Some(pos) => {
            // 检查填充长度至少为8字节
            if pos < 10 {
                return Err("PKCS#1填充格式错误：填充长度不足".to_string());
            }
            // 返回分隔符之后的数据
            Ok(decrypted_bytes[pos + 1..].to_vec())
        }
        None => Err("PKCS#1填充格式错误：未找到分隔符0x00".to_string())
    }
}

// 验证授权码
#[tauri::command]
pub async fn verify_license(
    license_code: String,
    app: AppHandle,
) -> Result<LicenseInfo, String> {
    // 1. 获取公钥
    let public_key_pem = {
        // 尝试从公钥管理器获取
        let manager_guard = PUBLIC_KEY_MANAGER.lock().await;
        if let Some(ref manager) = *manager_guard {
            manager.get_public_key().await
        } else {
            None
        }
    };
    
    // 如果还没有公钥，尝试从本地加载
    let public_key_pem = if let Some(key) = public_key_pem {
        key
    } else {
        PublicKeyManager::load_public_key_from_local(&app)
            .await
            .ok_or("未找到公钥，请等待公钥加载完成或检查网络连接".to_string())?
    };
    
    // 2. 使用 RSA 公钥解密授权码
    let license_str = decrypt_license_with_rsa(&license_code, &public_key_pem)?;

    // 3. 解析 JSON
    let license_info: LicenseInfo = serde_json::from_str(&license_str)
        .map_err(|_| "授权码解析失败（JSON格式错误）".to_string())?;
    
    // 验证工具ID
    if license_info.tool_id != "foldersync" {
        return Err("授权码工具ID不匹配".to_string());
    }
    
    // 验证有效期
    let expiry = chrono::DateTime::parse_from_rfc3339(&license_info.expiry_date)
        .map_err(|_| "授权码有效期格式错误".to_string())?
        .with_timezone(&chrono::Utc);
    
    let now = chrono::Utc::now();
    if expiry < now {
        return Err("授权码已过期".to_string());
    }
    
    // 保存授权信息（加密存储）
    save_license(&app, &license_code)?;
    
    Ok(license_info)
}

// 检查是否已注册
#[tauri::command]
pub async fn check_license_registered(app: AppHandle) -> Result<bool, String> {
    let license_path = get_license_file_path(&app);
    Ok(license_path.exists())
}

// 获取授权信息
#[tauri::command]
pub async fn get_license_info(app: AppHandle) -> Result<LicenseInfo, String> {
    let license_path = get_license_file_path(&app);
    
    if !license_path.exists() {
        return Err("未找到授权信息".to_string());
    }
    
    use base64::{Engine as _, engine::general_purpose};
    
    let encrypted = fs::read_to_string(&license_path)
        .map_err(|_| "读取授权信息失败".to_string())?;
    
    let decoded = general_purpose::STANDARD.decode(&encrypted)
        .map_err(|_| "解密授权信息失败".to_string())?;
    
    let license_code = String::from_utf8(decoded)
        .map_err(|_| "授权信息格式错误".to_string())?;
    
    // 重新验证授权码
    verify_license(license_code, app).await
}

// 检查授权码有效期
#[tauri::command]
pub async fn check_license_expiry(app: AppHandle) -> Result<i64, String> {
    let license_info = get_license_info(app).await?;
    
    let expiry = chrono::DateTime::parse_from_rfc3339(&license_info.expiry_date)
        .map_err(|_| "授权码有效期格式错误".to_string())?
        .with_timezone(&chrono::Utc);
    
    let now = chrono::Utc::now();
    let days_remaining = (expiry - now).num_days();
    
    Ok(days_remaining)
}

// 重置授权码（删除本地保存的授权信息）
#[tauri::command]
pub async fn reset_license(app: AppHandle) -> Result<(), String> {
    let license_path = get_license_file_path(&app);
    
    if license_path.exists() {
        fs::remove_file(&license_path)
            .map_err(|e| format!("删除授权文件失败: {}", e))?;
    }
    
    Ok(())
}

