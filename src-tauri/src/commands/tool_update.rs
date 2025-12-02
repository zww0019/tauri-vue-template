use crate::error::ApiResponse;
use crate::services::ToolUpdateService;
use crate::types::ToolUpdateInfo;
use std::sync::Arc;
use tauri::{Emitter, State};

/// 检查工具更新
#[tauri::command]
pub async fn check_tool_updates(
    tool_update_service: State<'_, Arc<tokio::sync::Mutex<ToolUpdateService>>>,
) -> Result<ApiResponse<Vec<ToolUpdateInfo>>, String> {
    let service = tool_update_service.lock().await;
    
        match service.check_for_updates().await {
            Ok(updates) => {
                let has_updates = !updates.is_empty();
                let updates_len = updates.len();
                let new_tools_count = updates.iter().filter(|u| u.current_version == "0.0.0").count();
                
                Ok(ApiResponse {
                    success: true,
                    data: Some(updates),
                    message: if has_updates {
                        format!("发现 {} 个可用更新 (其中 {} 个新工具)", updates_len, new_tools_count)
                    } else {
                        "所有工具已是最新版本".to_string()
                    },
                })
            }
            Err(e) => Ok(ApiResponse::error(e.to_string())),
        }
}

/// 下载并安装工具
#[tauri::command]
pub async fn update_tool(
    tool_id: String,
    download_url: String,
    checksum: String,
    tool_update_service: State<'_, Arc<tokio::sync::Mutex<ToolUpdateService>>>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<()>, String> {
    let mut service = tool_update_service.lock().await;
    
    // 下载工具
    let app_handle_clone = app_handle.clone();
    let tool_id_clone = tool_id.clone();
    let zip_path = service
        .download_tool(
            &tool_id,
            &download_url,
            Some(Box::new(move |downloaded, total, progress| {
                let _ = app_handle_clone.emit(
                    "tool:update-progress",
                    serde_json::json!({
                        "toolId": tool_id_clone,
                        "downloadedSize": downloaded,
                        "totalSize": total,
                        "progress": progress,
                    }),
                );
            })),
        )
        .await
        .map_err(|e| e.to_string())?;
    
    // 安装工具
    service
        .install_tool(&tool_id, &zip_path, &checksum)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(ApiResponse::success_with_message("工具更新成功", ()))
}

/// 批量更新工具
#[tauri::command]
pub async fn update_all_tools(
    updates: Vec<ToolUpdateInfo>,
    tool_update_service: State<'_, Arc<tokio::sync::Mutex<ToolUpdateService>>>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Vec<(String, bool, Option<String>)>>, String> {
    let mut service = tool_update_service.lock().await;
    let mut results = Vec::new();
    
    for update in updates {
        let tool_id = update.tool_id.clone();
        
        // 发送状态更新
        let _ = app_handle.emit(
            "tool:update-status",
            serde_json::json!({
                "toolId": tool_id,
                "status": "downloading",
            }),
        );
        
        match service
            .download_tool(
                &tool_id,
                &update.download_url,
                Some(Box::new({
                    let app_handle = app_handle.clone();
                    let tool_id = tool_id.clone();
                    move |downloaded, total, progress| {
                        let _ = app_handle.emit(
                            "tool:update-progress",
                            serde_json::json!({
                                "toolId": tool_id,
                                "downloadedSize": downloaded,
                                "totalSize": total,
                                "progress": progress,
                            }),
                        );
                    }
                })),
            )
            .await
        {
            Ok(zip_path) => {
                // 发送安装状态
                let _ = app_handle.emit(
                    "tool:update-status",
                    serde_json::json!({
                        "toolId": tool_id,
                        "status": "installing",
                    }),
                );
                
                match service.install_tool(&tool_id, &zip_path, &update.checksum).await {
                    Ok(_) => {
                        let _ = app_handle.emit(
                            "tool:update-status",
                            serde_json::json!({
                                "toolId": tool_id,
                                "status": "completed",
                            }),
                        );
                        results.push((tool_id, true, None));
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        let _ = app_handle.emit(
                            "tool:update-status",
                            serde_json::json!({
                                "toolId": tool_id,
                                "status": "failed",
                                "error": error_msg,
                            }),
                        );
                        results.push((tool_id, false, Some(error_msg)));
                    }
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                let _ = app_handle.emit(
                    "tool:update-status",
                    serde_json::json!({
                        "toolId": tool_id,
                        "status": "failed",
                        "error": error_msg,
                    }),
                );
                results.push((tool_id, false, Some(error_msg)));
            }
        }
    }
    
    Ok(ApiResponse {
        success: true,
        data: Some(results),
        message: "批量更新完成".to_string(),
    })
}

/// 清理备份
#[tauri::command]
pub async fn cleanup_tool_backups(
    tool_update_service: State<'_, Arc<tokio::sync::Mutex<ToolUpdateService>>>,
) -> Result<ApiResponse<()>, String> {
    let service = tool_update_service.lock().await;
    
    match service.cleanup_backups() {
        Ok(_) => Ok(ApiResponse::success_with_message("备份清理完成", ())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

