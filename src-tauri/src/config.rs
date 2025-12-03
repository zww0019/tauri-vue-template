use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// 读取服务器URL配置
// 优先级：环境变量 > 配置文件 > 默认值
pub fn get_server_url(app: &AppHandle) -> String {
    // 1. 首先尝试从环境变量读取
    if let Ok(url) = std::env::var("LICENSE_SERVER_URL") {
        println!("✓ 从环境变量读取服务器URL: {}", url);
        return url;
    }
    
    // 2. 尝试从配置文件读取
    let config_path = if let Ok(resource_dir) = app.path().resource_dir() {
        resource_dir.join("config.json")
    } else {
        // 开发环境下，尝试从源码目录读取
        PathBuf::from("src-tauri/config.json")
    };
    
    if config_path.exists() {
        if let Ok(config_content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_content) {
                if let Some(url) = config.get("license_server_url").and_then(|v| v.as_str()) {
                    println!("✓ 从配置文件读取服务器URL: {}", url);
                    return url.to_string();
                } else {
                    println!("⚠ 配置文件中未找到 license_server_url，使用默认值");
                }
            } else {
                println!("⚠ 配置文件格式错误，使用默认值");
            }
        } else {
            println!("⚠ 无法读取配置文件，使用默认值");
        }
    } else {
        println!("⚠ 配置文件不存在，使用默认服务器URL: https://api.example.com");
    }
    
    // 3. 使用默认值
    "https://api.example.com".to_string()
}

