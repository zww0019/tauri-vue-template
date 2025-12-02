use crate::error::ApiResponse;
use crate::services::{DependencyManager, ToolsManager};
use crate::types::{DependencyCheckResult, ToolInfo};
use std::sync::Arc;
use tauri::{Emitter, State};

/// 获取工具列表
#[tauri::command]
pub async fn get_tools_list(
    tools_manager: State<'_, Arc<tokio::sync::Mutex<ToolsManager>>>,
) -> Result<Vec<ToolInfo>, String> {
    let manager = tools_manager.lock().await;
    Ok(manager.get_tools_list())
}

/// 获取工具信息
#[tauri::command]
pub async fn get_tool_info(
    tool_id: String,
    tools_manager: State<'_, Arc<tokio::sync::Mutex<ToolsManager>>>,
) -> Result<Option<ToolInfo>, String> {
    let manager = tools_manager.lock().await;
    Ok(manager.get_tool_info(&tool_id))
}

/// 检查工具依赖状态
#[tauri::command]
pub async fn check_tool_dependencies(
    tool_id: String,
    tools_manager: State<'_, Arc<tokio::sync::Mutex<ToolsManager>>>,
    dependency_manager: State<'_, Arc<DependencyManager>>,
) -> Result<DependencyCheckResult, String> {
    let manager = tools_manager.lock().await;
    
    let tool = manager
        .get_tool_by_id(&tool_id)
        .or_else(|| manager.get_tool(&tool_id))
        .ok_or_else(|| format!("工具不存在: {}", tool_id))?;
    
    dependency_manager
        .check_dependencies_installed(&tool.manifest.id, &tool.manifest.dependencies)
        .map_err(|e| e.to_string())
}

/// 安装工具依赖
#[tauri::command]
pub async fn install_tool_dependencies(
    tool_id: String,
    tools_manager: State<'_, Arc<tokio::sync::Mutex<ToolsManager>>>,
    dependency_manager: State<'_, Arc<DependencyManager>>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<()>, String> {
    let manager = tools_manager.lock().await;
    
    let tool = manager
        .get_tool_by_id(&tool_id)
        .or_else(|| manager.get_tool(&tool_id))
        .ok_or_else(|| format!("工具不存在: {}", tool_id))?;
    
    if tool.manifest.dependencies.is_empty() {
        return Ok(ApiResponse::success_with_message("该工具无需依赖", ()));
    }
    
    // 发送进度更新的回调
    let app_handle_clone = app_handle.clone();
    let tool_id_clone = tool_id.clone();
    let progress_callback = Box::new(move |message: String, progress: f64| {
        let _ = app_handle_clone.emit(
            "tool:dependency-progress",
            serde_json::json!({
                "toolId": tool_id_clone,
                "message": message,
                "progress": progress,
            }),
        );
    });
    
    match dependency_manager
        .install_dependencies(
            &tool.manifest.id,
            &tool.manifest.dependencies,
            Some(progress_callback),
        )
        .await
    {
        Ok(_) => Ok(ApiResponse::success_with_message("依赖安装成功", ())),
        Err(e) => Ok(ApiResponse::error(format!("依赖安装失败: {}", e))),
    }
}

/// 重新加载工具列表
#[tauri::command]
pub async fn reload_tools(
    tools_manager: State<'_, Arc<tokio::sync::Mutex<ToolsManager>>>,
) -> Result<ApiResponse<()>, String> {
    let mut manager = tools_manager.lock().await;
    
    match manager.reload_tools() {
        Ok(_) => Ok(ApiResponse::success_with_message("工具列表已重新加载", ())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

