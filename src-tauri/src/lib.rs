mod error;
mod types;
mod services;
mod commands;

use services::{AuthService, DependencyManager, StorageService, ToolsManager};
use std::sync::Arc;
use tauri::Manager;

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
            
            // 4. 依赖管理服务
            let dependency_manager = Arc::new(
                DependencyManager::new(storage_service.clone())
            );
            log::info!("✓ 依赖管理服务初始化完成");
            
            // 注册状态管理
            app.manage(auth_service);
            app.manage(tools_manager);
            app.manage(dependency_manager);
            
            log::info!("应用服务初始化完成");
            
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
            commands::check_tool_dependencies,
            commands::install_tool_dependencies,
            commands::reload_tools,
            // 文件系统相关
            commands::select_file,
            commands::select_files,
            commands::select_folder,
            commands::save_file_dialog,
            commands::open_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
