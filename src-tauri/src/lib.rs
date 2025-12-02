mod error;
mod types;
mod services;
mod commands;
mod tool_plugin;

use services::{AuthService, StorageService, ToolsManager, ToolUpdateService, ToolExecutor};
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = tauri::Manager::get_webview_window(app, "main").unwrap();
                window.open_devtools();
            }
            
            // 初始化服务
            log::info!("初始化应用服务...");
            
            // 1. 存储服务
            let storage_service = Arc::new(
                StorageService::new()
                    .map_err(|e| format!("初始化存储服务失败: {}", e))?
            );
            log::info!("✓ 存储服务初始化完成");
            
            // 2. 认证服务
            let auth_service = Arc::new(tokio::sync::Mutex::new(
                AuthService::new(storage_service.clone())
            ));
            log::info!("✓ 认证服务初始化完成");
            
            // 3. 工具管理服务
            let tools_manager = Arc::new(tokio::sync::Mutex::new(
                ToolsManager::new(&app.handle())
                    .map_err(|e| format!("初始化工具管理服务失败: {}", e))?
            ));
            log::info!("✓ 工具管理服务初始化完成");
            
            // 4. 工具更新服务
            let tool_update_service = Arc::new(tokio::sync::Mutex::new(
                ToolUpdateService::new(&app.handle())
                    .map_err(|e| format!("初始化工具更新服务失败: {}", e))?
            ));
            log::info!("✓ 工具更新服务初始化完成");
            
            // 5. 工具执行服务
            let tool_executor = Arc::new(
                ToolExecutor::new(
                    tools_manager.clone(),
                    app.handle().clone(),
                )
            );
            log::info!("✓ 工具执行服务初始化完成");
            
            // 注册状态管理
            // 先克隆用于后台任务
            let auth_service_clone = auth_service.clone();
            app.manage(auth_service);
            app.manage(tools_manager.clone());
            app.manage(tool_update_service.clone());
            app.manage(tool_executor);
            
            log::info!("应用服务初始化完成");
            
            // 在后台自动获取公钥
            tauri::async_runtime::spawn(async move {
                // 等待一小段时间确保应用完全启动
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                log::info!("======== 开始自动获取公钥 ========");
                let mut auth = auth_service_clone.lock().await;
                
                // 检查是否已有公钥
                let has_public_key = auth.has_public_key();
                
                if has_public_key {
                    log::info!("已有本地公钥缓存，尝试获取最新版本...");
                } else {
                    log::info!("未找到本地公钥缓存，开始获取公钥...");
                }
                
                // 尝试获取公钥（带重试机制）
                let mut retry_count = 0;
                let max_retries = 5;
                
                while retry_count < max_retries {
                    match auth.fetch_public_key().await {
                        Ok(_) => {
                            log::info!("✓ 公钥获取成功 (尝试次数: {})", retry_count + 1);
                            break;
                        }
                        Err(e) => {
                            retry_count += 1;
                            if retry_count < max_retries {
                                // 计算重试延迟（指数退避：2^n * 1秒，最大60秒）
                                let delay_secs = std::cmp::min(
                                    (1 << std::cmp::min(retry_count - 1, 6)) as u64,
                                    60
                                );
                                log::warn!(
                                    "× 公钥获取失败 (第 {} 次尝试): {}，{}秒后重试...",
                                    retry_count,
                                    e,
                                    delay_secs
                                );
                                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                            } else {
                                log::error!("× 公钥获取失败，已达到最大重试次数 ({} 次)", max_retries);
                                if !has_public_key {
                                    log::warn!("警告：未获取到公钥，许可证验证功能可能不可用");
                                }
                            }
                        }
                    }
                }
                log::info!("======== 公钥获取流程完成 ========");
            });
            
            // 在后台自动检查并拉取工具（仅在打包后的程序中执行，开发环境跳过）
            // 使用 Tauri 的异步运行时，它会在应用启动后运行
            #[cfg(not(debug_assertions))]
            {
                let app_handle = app.handle().clone();
                let tools_manager_clone = tools_manager.clone();
                let tool_update_service_clone = tool_update_service.clone();
                
                // 使用 Tauri 的异步运行时启动后台任务
                tauri::async_runtime::spawn(async move {
                    // 等待一小段时间确保应用完全启动
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    
                    log::info!("======== 开始自动检查工具更新 ========");
                
                // 先检查更新，使用作用域确保锁在检查完成后释放
                let updates = {
                    let update_service = tool_update_service_clone.lock().await;
                    match update_service.check_for_updates().await {
                        Ok(updates) => updates,
                        Err(e) => {
                            log::warn!("检查工具更新失败: {}，将在下次启动时重试", e);
                            return;
                        }
                    }
                };
                
                if !updates.is_empty() {
                    log::info!("发现 {} 个可用更新，开始自动下载", updates.len());
                    
                    // 自动下载并安装所有新工具和更新
                    let mut update_service = tool_update_service_clone.lock().await;
                    for update in updates {
                        let tool_id = update.tool_id.clone();
                        log::info!("正在更新工具: {}", tool_id);
                        
                        // 发送开始下载状态
                        let _ = app_handle.emit(
                            "tool:update-status",
                            serde_json::json!({
                                "toolId": tool_id,
                                "status": "downloading",
                            }),
                        );
                        
                        match update_service
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
                                
                                match update_service
                                    .install_tool(&tool_id, &zip_path, &update.checksum)
                                    .await
                                {
                                    Ok(_) => {
                                        log::info!("✓ 工具 {} 更新成功", tool_id);
                                        
                                        // 重新加载工具列表
                                        let mut manager = tools_manager_clone.lock().await;
                                        if let Err(e) = manager.reload_tools() {
                                            log::error!("重新加载工具失败: {}", e);
                                        }
                                        
                                        // 发送完成状态
                                        let _ = app_handle.emit(
                                            "tool:update-status",
                                            serde_json::json!({
                                                "toolId": tool_id,
                                                "status": "completed",
                                            }),
                                        );
                                        
                                        // 发送事件通知前端
                                        let _ = app_handle.emit(
                                            "tool:auto-updated",
                                            serde_json::json!({
                                                "toolId": tool_id,
                                                "version": update.remote_version,
                                            }),
                                        );
                                    }
                                    Err(e) => {
                                        log::error!("✗ 工具 {} 安装失败: {}", tool_id, e);
                                        
                                        // 发送失败状态
                                        let _ = app_handle.emit(
                                            "tool:update-status",
                                            serde_json::json!({
                                                "toolId": tool_id,
                                                "status": "failed",
                                                "error": e.to_string(),
                                            }),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("✗ 工具 {} 下载失败: {}", tool_id, e);
                                
                                // 发送失败状态
                                let _ = app_handle.emit(
                                    "tool:update-status",
                                    serde_json::json!({
                                        "toolId": tool_id,
                                        "status": "failed",
                                        "error": e.to_string(),
                                    }),
                                );
                            }
                        }
                    }
                    
                    log::info!("======== 工具自动更新完成 ========");
                } else {
                    log::info!("所有工具已是最新版本");
                }
                });
            }
            
            #[cfg(debug_assertions)]
            {
                log::info!("开发环境：跳过自动工具更新检查");
            }
            
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![
            // 认证相关
            commands::fetch_public_key,
            commands::register_tool_license,
            commands::check_tool_auth_status,
            commands::get_registered_tools,
            commands::reset_tool_auth,
            // 工具相关
            commands::get_tools_list,
            commands::get_tool_info,
            commands::reload_tools,
            // 工具更新相关
            commands::check_tool_updates,
            commands::update_tool,
            commands::update_all_tools,
            commands::cleanup_tool_backups,
            // 文件系统相关
            commands::select_file,
            commands::select_files,
            commands::select_folder,
            commands::save_file_dialog,
            commands::open_path,
            // 工具执行相关
            commands::execute_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
