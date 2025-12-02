use crate::error::ApiResponse;
use crate::services::AuthService;
use crate::types::AuthStatus;
use std::sync::Arc;
use tauri::State;

/// 从服务器获取公钥
#[tauri::command]
pub async fn fetch_public_key(
    auth_service: State<'_, Arc<tokio::sync::Mutex<AuthService>>>,
) -> Result<ApiResponse<String>, String> {
    let mut auth = auth_service.lock().await;
    
    match auth.fetch_public_key().await {
        Ok(key) => Ok(ApiResponse::success(key)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 注册工具License
#[tauri::command]
pub async fn register_tool_license(
    tool_id: String,
    license_code: String,
    auth_service: State<'_, Arc<tokio::sync::Mutex<AuthService>>>,
) -> Result<ApiResponse<()>, String> {
    let auth = auth_service.lock().await;
    
    match auth.register_tool_license(&tool_id, &license_code).await {
        Ok(response) => Ok(response),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 检查工具认证状态
#[tauri::command]
pub async fn check_tool_auth_status(
    tool_id: String,
    auth_service: State<'_, Arc<tokio::sync::Mutex<AuthService>>>,
) -> Result<AuthStatus, String> {
    let auth = auth_service.lock().await;
    
    match auth.check_tool_auth_status(&tool_id).await {
        Ok(status) => Ok(status),
        Err(e) => {
            log::error!("检查认证状态失败: {}", e);
            Ok(AuthStatus {
                is_authenticated: false,
                license_data: None,
            })
        }
    }
}

/// 获取所有已注册的工具
#[tauri::command]
pub async fn get_registered_tools(
    auth_service: State<'_, Arc<tokio::sync::Mutex<AuthService>>>,
) -> Result<Vec<String>, String> {
    let auth = auth_service.lock().await;
    
    match auth.get_registered_tools().await {
        Ok(tools) => Ok(tools),
        Err(e) => {
            log::error!("获取已注册工具失败: {}", e);
            Ok(Vec::new())
        }
    }
}

/// 重置工具认证
#[tauri::command]
pub async fn reset_tool_auth(
    tool_id: Option<String>,
    auth_service: State<'_, Arc<tokio::sync::Mutex<AuthService>>>,
) -> Result<ApiResponse<()>, String> {
    let auth = auth_service.lock().await;
    
    match auth.reset_tool_auth(tool_id.as_deref()) {
        Ok(_) => Ok(ApiResponse::success_with_message("重置成功", ())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

