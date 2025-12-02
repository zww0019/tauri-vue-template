/// 选择文件
#[tauri::command]
pub async fn select_file(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file_path = app_handle
        .dialog()
        .file()
        .blocking_pick_file();
    
    Ok(file_path.map(|p| p.to_string()))
}

/// 选择多个文件
#[tauri::command]
pub async fn select_files(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file_paths = app_handle
        .dialog()
        .file()
        .blocking_pick_files();
    
    Ok(file_paths
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect())
}

/// 选择文件夹
#[tauri::command]
pub async fn select_folder(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let folder_path = app_handle
        .dialog()
        .file()
        .blocking_pick_folder();
    
    Ok(folder_path.map(|p| p.to_string()))
}

/// 保存文件对话框
#[tauri::command]
pub async fn save_file_dialog(
    app_handle: tauri::AppHandle,
    default_name: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let mut dialog = app_handle.dialog().file();
    
    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }
    
    let file_path = dialog.blocking_save_file();
    
    Ok(file_path.map(|p| p.to_string()))
}

/// 在系统文件管理器中打开路径
#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let command = "explorer";
    
    #[cfg(target_os = "macos")]
    let command = "open";
    
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    
    std::process::Command::new(command)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("打开路径失败: {}", e))?;
    
    Ok(())
}

