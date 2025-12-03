// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{Manager, Emitter};
use tauri::menu::{MenuBuilder, Submenu, MenuItem, PredefinedMenuItem};

// 模块声明
mod license;
mod public_key;
mod update;
mod config;

// 重新导出常用类型
pub use license::LicenseInfo;

// 导入模块中的命令
use license::{
    check_license_expiry, check_license_registered, get_license_info, verify_license, reset_license,
};
use public_key::{get_public_key, init_public_key_manager, refresh_public_key};
use update::{check_update, download_update, install_update};
use config::get_server_url;

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Backend was called with an argument: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            
            // 创建菜单
            // 应用菜单（macOS）
            #[cfg(target_os = "macos")]
            let app_menu = Submenu::with_items(
                app,
                "tauri-app",
                true,
                &[
                    &PredefinedMenuItem::about(app, Some("关于"), None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::services(app, Some("服务"))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, Some("隐藏"))?,
                    &PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?,
                    &PredefinedMenuItem::show_all(app, Some("显示全部"))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, Some("退出"))?,
                ],
            )?;
            
            // 授权菜单
            let reset_license_item = MenuItem::with_id(
                app,
                "reset_license",
                "重置授权码",
                true,
                None::<&str>,
            )?;
            
            let license_menu = Submenu::with_items(
                app,
                "授权",
                true,
                &[&reset_license_item],
            )?;
            
            // 编辑菜单
            let edit_menu = Submenu::with_items(
                app,
                "编辑",
                true,
                &[
                    &PredefinedMenuItem::undo(app, Some("撤销"))?,
                    &PredefinedMenuItem::redo(app, Some("重做"))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, Some("剪切"))?,
                    &PredefinedMenuItem::copy(app, Some("复制"))?,
                    &PredefinedMenuItem::paste(app, Some("粘贴"))?,
                    &PredefinedMenuItem::select_all(app, Some("全选"))?,
                ],
            )?;
            
            // 窗口菜单
            let window_menu = Submenu::with_items(
                app,
                "窗口",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, Some("最小化"))?,
                    &PredefinedMenuItem::maximize(app, Some("最大化"))?,
                    &PredefinedMenuItem::close_window(app, Some("关闭窗口"))?,
                ],
            )?;
            
            // 构建菜单栏
            #[cfg(target_os = "macos")]
            let menu = MenuBuilder::new(app)
                .item(&app_menu)
                .item(&license_menu)
                .item(&edit_menu)
                .item(&window_menu)
                .build()?;
            
            #[cfg(not(target_os = "macos"))]
            let menu = MenuBuilder::new(app)
                .item(&license_menu)
                .item(&edit_menu)
                .item(&window_menu)
                .build()?;
            
            app.set_menu(menu)?;
            
            // 处理菜单事件
            let app_handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                if event.id() == "reset_license" {
                    // 发送事件到前端
                    let _ = app_handle.emit("menu-reset-license", ());
                }
            });
            
            // 初始化公钥管理器
            let server_url = get_server_url(&app.handle());
            let app_handle = app.handle().clone();
            
            // 使用 Tauri 的异步运行时
            tauri::async_runtime::spawn(async move {
                init_public_key_manager(server_url, app_handle).await;
            });
            
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            verify_license,
            check_license_registered,
            get_license_info,
            check_license_expiry,
            reset_license,
            check_update,
            download_update,
            install_update,
            refresh_public_key,
            get_public_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
