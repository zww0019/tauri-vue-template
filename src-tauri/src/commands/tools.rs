use crate::error::ApiResponse;
use crate::services::ToolsManager;
use crate::types::ToolInfo;
use std::sync::Arc;
use tauri::State;

/// 获取工具列表
#[tauri::command]
pub async fn get_tools_list(
    tools_manager: State<'_, Arc<tokio::sync::Mutex<ToolsManager>>>,
) -> Result<Vec<ToolInfo>, String> {
    let manager = tools_manager.lock().await;
    let tools_list = manager.get_tools_list();
    log::info!("get_tools_list 返回 {} 个工具", tools_list.len());
    for tool in &tools_list {
        log::info!("  - {} (id: {})", tool.name, tool.id);
    }
    Ok(tools_list)
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

