use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: String,
    pub required_auth: bool,
    pub has_dependencies: bool,
    pub hidden: bool,
}

/// 工具清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub main: String,
    pub icon: Option<String>,
    #[serde(rename = "requiredAuth", default = "default_required_auth")]
    pub required_auth: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

fn default_required_auth() -> bool {
    true
}

/// License数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    #[serde(rename = "toolId")]
    pub tool_id: String,
    pub username: String,
    #[serde(rename = "expiryDate")]
    pub expiry_date: String,
    #[serde(rename = "maxDevices")]
    pub max_devices: i32,
}

/// 工具License信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolLicense {
    pub license_code: String,
    pub license_data: LicenseData,
    pub device_fingerprint: String,
    pub registered_date: String,
    pub server_verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<String>,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub platform: String,
    pub arch: String,
    pub app_version: String,
}

/// 认证状态
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub license_data: Option<LicenseData>,
}

/// 依赖检查结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyCheckResult {
    pub installed: bool,
    pub missing: Vec<String>,
    pub details: HashMap<String, bool>,
}

/// 进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i32>,
}

/// 工具执行参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolExecuteArgs {
    pub tool_name: String,
    pub method: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

/// 工具更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUpdateInfo {
    pub tool_id: String,
    pub current_version: String,
    pub remote_version: String,
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
    pub release_notes: String,
}

/// 工具版本配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolVersionConfig {
    pub version: String,
    #[serde(rename = "updateUrl")]
    pub update_url: String,
    pub tools: HashMap<String, RemoteToolInfo>,
}

/// 远程工具信息
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteToolInfo {
    pub version: String,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    pub checksum: String,
    pub size: u64,
    #[serde(rename = "releaseNotes")]
    pub release_notes: String,
}

