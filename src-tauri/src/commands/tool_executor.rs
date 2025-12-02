use crate::error::ApiResponse;
use crate::services::ToolExecutor;
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

/// 执行工具方法
#[tauri::command]
pub async fn execute_tool(
    tool_id: String,
    method: String,
    args: Value,
    tool_executor: State<'_, Arc<ToolExecutor>>,
) -> Result<ApiResponse<Value>, String> {
    log::info!("收到 execute_tool 命令: tool_id={}, method={}", tool_id, method);
    match tool_executor.execute_tool(&tool_id, &method, args).await {
        Ok(result) => {
            log::info!("execute_tool 执行成功");
            Ok(ApiResponse::success(result))
        }
        Err(e) => {
            log::error!("execute_tool 执行失败: {}", e);
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

