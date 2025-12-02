use crate::error::{AppError, AppResult};
use crate::services::{tools::ToolsManager, tool_loader::ToolLoader};
use toolset_plugin::{ToolPlugin, ProgressData};
use serde_json::Value;
use std::sync::Arc;
use tauri::Emitter;

/// 工具执行服务 - 负责执行 Rust 插件工具
pub struct ToolExecutor {
    tools_manager: Arc<tokio::sync::Mutex<ToolsManager>>,
    app_handle: tauri::AppHandle,
}

impl ToolExecutor {
    pub fn new(
        tools_manager: Arc<tokio::sync::Mutex<ToolsManager>>,
        app_handle: tauri::AppHandle,
    ) -> Self {
        Self {
            tools_manager,
            app_handle,
        }
    }

    /// 执行工具方法
    pub async fn execute_tool(
        &self,
        tool_id: &str,
        method: &str,
        args: Value,
    ) -> AppResult<Value> {
        log::info!("执行工具方法: tool_id={}, method={}, args={:?}", tool_id, method, args);
        
        // 获取工具插件
        let plugin = {
            let manager = self.tools_manager.lock().await;
            
            // 先尝试获取已加载的插件
            if let Some(plugin) = manager.get_tool_plugin(tool_id) {
                log::info!("使用已加载的插件: {}", tool_id);
                plugin
            } else {
                log::info!("插件未加载，尝试加载: {}", tool_id);
                // 如果插件未加载，尝试加载
                let tool = manager.get_tool_by_id(tool_id)
                    .or_else(|| manager.get_tool(tool_id))
                    .ok_or_else(|| {
                        log::error!("工具不存在: {}", tool_id);
                        AppError::Tool(format!("工具不存在: {}", tool_id))
                    })?;
                
                log::info!("工具路径: {:?}, 插件路径: {:?}", tool.path, tool.plugin_path);
                
                // 尝试加载插件
                if tool.plugin_path.exists() {
                    unsafe {
                        match ToolLoader::load_plugin(&tool.plugin_path) {
                            Ok(p) => {
                                log::info!("运行时加载工具 {} 插件成功", tool_id);
                                // 保存插件到工具管理器，避免重复加载
                                let plugin_clone = p.clone();
                                drop(manager); // 释放锁
                                if let Err(e) = self.tools_manager.lock().await.set_tool_plugin(tool_id, plugin_clone) {
                                    log::warn!("保存插件到工具管理器失败: {}，但继续使用", e);
                                }
                                p
                            }
                            Err(e) => {
                                log::error!("加载工具插件失败: {}", e);
                                return Err(AppError::Tool(format!(
                                    "加载工具插件失败: {}",
                                    e
                                )));
                            }
                        }
                    }
                } else {
                    log::error!("工具插件文件不存在: {:?}", tool.plugin_path);
                    return Err(AppError::Tool(format!(
                        "工具插件文件不存在: {:?}",
                        tool.plugin_path
                    )));
                }
            }
        };

        // 创建进度回调
        let app_handle = self.app_handle.clone();
        let tool_id_clone = tool_id.to_string();
        let on_progress = Box::new(move |progress: ProgressData| {
            eprintln!("[tool_executor] 收到进度回调: tool_id={}, task_id={}, status={}, has_log={}", 
                tool_id_clone, 
                progress.task_id, 
                progress.status,
                progress.log.is_some()
            );
            
            let mut progress_json = serde_json::json!({
                "toolId": tool_id_clone,
                "taskId": progress.task_id,
                "status": progress.status,
                "message": progress.message,
                "progress": progress.progress,
                "processedFiles": progress.processed_files,
                "totalFiles": progress.total_files,
                "processedSize": progress.processed_size,
                "totalSize": progress.total_size,
            });
            
            // 如果有日志，添加到事件中
            if let Some(log) = progress.log {
                eprintln!("[tool_executor] 发送日志事件: task_id={}, message={}", progress.task_id, log.message);
                progress_json["log"] = serde_json::json!({
                    "time": log.time,
                    "type": log.level,
                    "message": log.message,
                });
            }
            
            let emit_result = app_handle.emit("tool:execution-progress", progress_json);
            if let Err(e) = emit_result {
                eprintln!("[tool_executor] 发送事件失败: {}", e);
            } else {
                eprintln!("[tool_executor] 事件发送成功");
            }
        });

        // 执行工具方法
        log::info!("调用插件 execute 方法: method={}", method);
        let result = plugin
            .execute(method, args, on_progress)
            .map_err(|e| {
                log::error!("工具执行失败: {}", e);
                AppError::Tool(format!("工具执行失败: {}", e))
            })?;

        log::info!("工具执行成功，返回结果: {:?}", result);
        Ok(result)
    }

    /// 停止任务
    pub async fn stop_task(&self, tool_id: &str, task_id: &str) -> AppResult<()> {
        let plugin = {
            let manager = self.tools_manager.lock().await;
            manager
                .get_tool_plugin(tool_id)
                .ok_or_else(|| AppError::Tool(format!("工具不存在或插件未加载: {}", tool_id)))?
        };

        plugin
            .stop_task(task_id)
            .map_err(|e| AppError::Tool(format!("停止任务失败: {}", e)))?;

        Ok(())
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, tool_id: &str, task_id: &str) -> AppResult<Value> {
        let plugin = {
            let manager = self.tools_manager.lock().await;
            manager
                .get_tool_plugin(tool_id)
                .ok_or_else(|| AppError::Tool(format!("工具不存在或插件未加载: {}", tool_id)))?
        };

        let status = plugin
            .get_task_status(task_id)
            .map_err(|e| AppError::Tool(format!("获取任务状态失败: {}", e)))?;

        Ok(serde_json::json!({
            "taskId": status.task_id,
            "status": status.status,
            "message": status.message,
            "progress": status.progress,
        }))
    }

    /// 获取任务日志
    pub async fn get_task_logs(&self, tool_id: &str, task_id: &str, limit: usize) -> AppResult<Value> {
        let plugin = {
            let manager = self.tools_manager.lock().await;
            manager
                .get_tool_plugin(tool_id)
                .ok_or_else(|| AppError::Tool(format!("工具不存在或插件未加载: {}", tool_id)))?
        };

        let logs = plugin
            .get_task_logs(task_id, limit)
            .map_err(|e| AppError::Tool(format!("获取任务日志失败: {}", e)))?;

        let logs_json: Vec<Value> = logs
            .into_iter()
            .map(|log| {
                serde_json::json!({
                    "time": log.time,
                    "level": log.level,
                    "message": log.message,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "logs": logs_json,
        }))
    }
}
