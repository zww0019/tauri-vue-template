//! FolderSync 工具 - Rust 实现
//! 
//! 单向文件夹实时同步工具，支持多个文件夹同时监控

use toolset_plugin::{ToolPlugin, ToolInfo, ProgressData, TaskStatus, LogEntry};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::mpsc;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use walkdir::WalkDir;
use chrono::Utc;
use uuid::Uuid;

/// 同步任务状态
#[derive(Debug, Clone, PartialEq)]
enum SyncStatus {
    Initializing,
    Syncing,
    Stopped,
    Error(String),
}

/// 同步任务
struct SyncTask {
    id: String,
    source_dir: PathBuf,
    target_dir: PathBuf,
    status: SyncStatus,
    logs: Vec<toolset_plugin::LogEntry>,
    cancelled: Arc<Mutex<bool>>,
    watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
}

/// FolderSync 工具实现
pub struct FolderSyncTool {
    tasks: Arc<Mutex<HashMap<String, SyncTask>>>,
    task_counter: Arc<Mutex<u64>>,
    // 保持运行时存活（仅在无法获取当前运行时的情况下使用）
    #[allow(dead_code)]
    runtime: Option<tokio::runtime::Runtime>,
    // 任务进度回调映射
    progress_callbacks: Arc<Mutex<HashMap<String, Box<dyn Fn(toolset_plugin::ProgressData) + Send>>>>,
}

impl FolderSyncTool {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            task_counter: Arc::new(Mutex::new(0)),
            runtime: None,
            progress_callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加日志（并发送事件）
    fn add_log(&self, task_id: &str, level: &str, message: &str) {
        // 直接输出到 stderr（绕过日志系统，确保能看到）
        eprintln!("[foldersync][{}][{}] {}", task_id, level, message);
        
        // 同时输出到标准日志（如果已初始化）
        match level {
            "error" => log::error!("[{}] {}", task_id, message),
            "warn" => log::warn!("[{}] {}", task_id, message),
            _ => log::info!("[{}] {}", task_id, message),
        }
        
        let log = toolset_plugin::LogEntry {
            time: Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
        };
        
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            task.logs.push(log.clone());
            
            // 限制日志数量
            if task.logs.len() > 10000 {
                task.logs = task.logs.split_off(task.logs.len() - 10000);
            }
            
            // 通过事件发送日志
            let callbacks = self.progress_callbacks.lock().unwrap();
            if let Some(on_progress) = callbacks.get(task_id) {
                eprintln!("[foldersync] 找到回调，发送日志事件: task_id={}, message={}", task_id, message);
                on_progress(toolset_plugin::ProgressData {
                    task_id: task_id.to_string(),
                    status: match task.status {
                        SyncStatus::Initializing => "initializing".to_string(),
                        SyncStatus::Syncing => "syncing".to_string(),
                        SyncStatus::Stopped => "stopped".to_string(),
                        SyncStatus::Error(_) => "error".to_string(),
                    },
                    message: None,
                    progress: None,
                    processed_files: None,
                    total_files: None,
                    processed_size: None,
                    total_size: None,
                    log: Some(log.clone()), // 发送日志事件
                });
            } else {
                eprintln!("[foldersync] 警告：未找到回调，无法发送日志事件: task_id={}", task_id);
            }
        } else {
            eprintln!("[foldersync] 警告：任务不存在: task_id={}", task_id);
        }
    }

    /// 格式化文件大小
    fn format_size(&self, bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// 初始化同步任务
    async fn initialize_sync(
        &self,
        task_id: &str,
    ) -> Result<(), String> {
        eprintln!("[foldersync] ======== initialize_sync 开始: task_id={} ========", task_id);
        log::info!("======== initialize_sync 开始: task_id={} ========", task_id);
        
        // 获取任务路径信息（不克隆整个任务）
        let (source_dir, target_dir, cancelled) = {
            let tasks = self.tasks.lock().unwrap();
            let task = tasks.get(task_id)
                .ok_or_else(|| {
                    log::error!("任务不存在: {}", task_id);
                    "任务不存在".to_string()
                })?;
            log::info!("获取任务路径: 源={:?}, 目标={:?}", task.source_dir, task.target_dir);
            (task.source_dir.clone(), task.target_dir.clone(), task.cancelled.clone())
        };

        log::info!("添加日志: 开始初始化同步（对齐文件夹）...");
        self.add_log(task_id, "info", "开始初始化同步（对齐文件夹）...");

        // 统计文件
        let mut total_files = 0;
        let mut total_size = 0u64;

        for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                total_files += 1;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        self.add_log(
            task_id,
            "info",
            &format!("发现 {} 个文件，总大小: {}", total_files, self.format_size(total_size)),
        );

        // 执行同步
        let mut processed_files = 0;
        let mut processed_size = 0u64;

        for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
            if *cancelled.lock().unwrap() {
                return Err("任务已取消".to_string());
            }

            let source_path = entry.path();
            let relative = source_path.strip_prefix(&source_dir)
                .map_err(|e| format!("路径错误: {}", e))?;
            let target_path = target_dir.join(relative);

            if entry.file_type().is_dir() {
                // 创建目录
                fs::create_dir_all(&target_path).await
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                // 检查是否需要同步
                let need_sync = if target_path.exists() {
                    let source_meta = fs::metadata(source_path).await
                        .map_err(|e| format!("获取源文件元数据失败: {}", e))?;
                    let target_meta = fs::metadata(&target_path).await
                        .map_err(|e| format!("获取目标文件元数据失败: {}", e))?;
                    
                    // 比较文件大小和修改时间
                    let size_diff = source_meta.len() != target_meta.len();
                    let time_diff = source_meta.modified()
                        .and_then(|s| target_meta.modified().map(|t| s != t))
                        .unwrap_or(true);
                    
                    size_diff || time_diff
                } else {
                    true
                };

                if need_sync {
                    fs::copy(source_path, &target_path).await
                        .map_err(|e| format!("复制文件失败: {}", e))?;
                    
                    let relative_str = relative.to_string_lossy();
                    self.add_log(task_id, "sync", &format!("已同步: {}", relative_str));
                }

                processed_files += 1;
                if let Ok(metadata) = entry.metadata() {
                    processed_size += metadata.len();
                }

                // 更新进度
                if total_files > 0 {
                    let progress = (processed_files as f64 / total_files as f64) * 100.0;
                    let callbacks = self.progress_callbacks.lock().unwrap();
                    if let Some(on_progress) = callbacks.get(task_id) {
                        on_progress(ProgressData {
                            task_id: task_id.to_string(),
                            status: "initializing".to_string(),
                            message: Some(format!("初始化同步中: {}/{} 文件", processed_files, total_files)),
                            progress: Some(progress),
                            processed_files: Some(processed_files),
                            total_files: Some(total_files),
                            processed_size: Some(processed_size),
                            total_size: Some(total_size),
                            log: None,
                        });
                    }
                }
            }
        }

        self.add_log(
            task_id,
            "success",
            &format!("初始化同步完成！共处理 {} 个文件", processed_files),
        );

        // 更新任务状态
        {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = SyncStatus::Syncing;
                log::info!("任务状态已更新为: Syncing");
            }
        }

        log::info!("======== initialize_sync 完成: task_id={} ========", task_id);
        Ok(())
    }

    /// 启动文件监听
    fn start_watching(&self, task_id: &str) -> Result<(), String> {
        eprintln!("[foldersync] ======== start_watching 被调用: task_id={} ========", task_id);
        log::info!("======== start_watching 被调用: task_id={} ========", task_id);
        
        // 获取任务路径信息和 watcher 引用
        let (source_dir, target_dir, cancelled, watcher_arc) = {
            let tasks = self.tasks.lock().unwrap();
            let task = tasks.get(task_id)
                .ok_or_else(|| {
                    log::error!("任务不存在: {}", task_id);
                    "任务不存在".to_string()
                })?;
            log::info!("获取任务信息: 源目录={:?}, 目标目录={:?}", task.source_dir, task.target_dir);
            (
                task.source_dir.clone(),
                task.target_dir.clone(),
                task.cancelled.clone(),
                task.watcher.clone(),
            )
        };

        log::info!("添加日志: 开始监听文件夹变化...");
        self.add_log(task_id, "info", "开始监听文件夹变化...");

        // 创建文件监听器
        let (tx, mut rx) = mpsc::unbounded_channel();
        let tx_clone = tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    log::info!("notify 回调收到事件: {:?}, 路径: {:?}", event.kind, event.paths);
                    if let Err(e) = tx_clone.send(event) {
                        log::error!("发送事件到 channel 失败: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("notify 回调错误: {}", e);
                }
            }
        }).map_err(|e| format!("创建文件监听器失败: {}", e))?;

        // 调用 watch 开始监听
        watcher.watch(&source_dir, RecursiveMode::Recursive)
            .map_err(|e| {
                log::error!("开始监听失败: {}, 目录: {:?}", e, source_dir);
                format!("开始监听失败: {}", e)
            })?;
        
        log::info!("已开始监听目录: {:?}", source_dir);
        
        // 将 watcher 保存到任务结构体中，确保它在任务存活期间保持存活
        // 注意：必须在 watch 调用之后保存，否则 watcher 会被丢弃
        {
            let mut watcher_guard = watcher_arc.lock().unwrap();
            *watcher_guard = Some(watcher);
        }
        
        log::info!("Watcher 已保存到任务结构体中");

        // 启动异步任务处理文件变化
        let task_id_clone = task_id.to_string();
        let source_dir_clone = source_dir.clone();
        let target_dir_clone = target_dir.clone();
        let tasks_clone = self.tasks.clone();
        let progress_callbacks_clone = self.progress_callbacks.clone();

        // 获取当前 Tokio 运行时句柄，如果不存在则创建新的运行时
        // 注意：在动态库中，我们需要确保在运行时上下文中调用
        let handle = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                // 如果无法获取当前运行时，创建一个新的并保持存活
                // 注意：这应该很少发生，因为 Tauri 已经提供了运行时
                log::warn!("无法获取当前 Tokio 运行时，创建新的运行时");
                tokio::runtime::Runtime::new()
                    .expect("无法创建 Tokio 运行时")
                    .handle()
                    .clone()
            }
        };

        handle.spawn(async move {
            log::info!("文件监听任务已启动，等待文件变化... (task_id: {})", task_id_clone);
            log::info!("监听源目录: {:?}", source_dir_clone);
            log::info!("目标目录: {:?}", target_dir_clone);
            
            // 测试：发送一个测试消息确认 channel 工作
            log::info!("Channel 接收端已就绪，等待事件...");
            
            while let Some(event) = rx.recv().await {
                if *cancelled.lock().unwrap() {
                    log::info!("任务已取消，停止监听");
                    break;
                }

                log::info!("收到文件系统事件: {:?}, 路径: {:?}", event.kind, event.paths);

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for path in event.paths {
                            let relative_opt = path.strip_prefix(&source_dir_clone)
                                .map(|r| r.to_path_buf());
                            
                            if let Ok(relative) = relative_opt {
                                let target_path = target_dir_clone.join(&relative);
                                let path_clone = path.clone();
                                let relative_str = relative.display().to_string();
                                let task_id_for_log = task_id_clone.clone();
                                let tasks_for_log = tasks_clone.clone();
                                let callbacks_for_log = progress_callbacks_clone.clone();
                                
                                tokio::spawn(async move {
                                    // 延迟一下，确保文件写入完成
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                    
                                    if let Ok(metadata) = fs::metadata(&path_clone).await {
                                        let log_entry = if metadata.is_dir() {
                                            let _ = fs::create_dir_all(&target_path).await;
                                            toolset_plugin::LogEntry {
                                                time: Utc::now().to_rfc3339(),
                                                level: "sync".to_string(),
                                                message: format!("创建目录: {}", relative_str),
                                            }
                                        } else {
                                            let success = fs::copy(&path_clone, &target_path).await.is_ok();
                                            toolset_plugin::LogEntry {
                                                time: Utc::now().to_rfc3339(),
                                                level: if success { "sync".to_string() } else { "error".to_string() },
                                                message: if success {
                                                    format!("同步文件: {}", relative_str)
                                                } else {
                                                    format!("同步文件失败: {}", relative_str)
                                                },
                                            }
                                        };
                                        
                                        // 添加日志并发送事件
                                        let mut tasks = tasks_for_log.lock().unwrap();
                                        if let Some(task) = tasks.get_mut(&task_id_for_log) {
                                            task.logs.push(log_entry.clone());
                                            
                                            // 限制日志数量
                                            if task.logs.len() > 10000 {
                                                task.logs = task.logs.split_off(task.logs.len() - 10000);
                                            }
                                            
                                            // 通过事件发送日志
                                            let callbacks = callbacks_for_log.lock().unwrap();
                                            if let Some(on_progress) = callbacks.get(&task_id_for_log) {
                                                on_progress(toolset_plugin::ProgressData {
                                                    task_id: task_id_for_log.clone(),
                                                    status: "syncing".to_string(),
                                                    message: None,
                                                    progress: None,
                                                    processed_files: None,
                                                    total_files: None,
                                                    processed_size: None,
                                                    total_size: None,
                                                    log: Some(log_entry),
                                                });
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            let relative_opt = path.strip_prefix(&source_dir_clone)
                                .map(|r| r.to_path_buf());
                            
                            if let Ok(relative) = relative_opt {
                                let target_path = target_dir_clone.join(&relative);
                                let relative_str = relative.display().to_string();
                                let task_id_for_log = task_id_clone.clone();
                                let tasks_for_log = tasks_clone.clone();
                                let callbacks_for_log = progress_callbacks_clone.clone();
                                
                                tokio::spawn(async move {
                                    if target_path.exists() {
                                        let is_dir = target_path.is_dir();
                                        let success = if is_dir {
                                            fs::remove_dir_all(&target_path).await.is_ok()
                                        } else {
                                            fs::remove_file(&target_path).await.is_ok()
                                        };
                                        
                                        let log_entry = toolset_plugin::LogEntry {
                                            time: Utc::now().to_rfc3339(),
                                            level: if success { "sync".to_string() } else { "error".to_string() },
                                            message: if success {
                                                format!("删除: {}", relative_str)
                                            } else {
                                                format!("删除失败: {}", relative_str)
                                            },
                                        };
                                        
                                        // 添加日志并发送事件
                                        let mut tasks = tasks_for_log.lock().unwrap();
                                        if let Some(task) = tasks.get_mut(&task_id_for_log) {
                                            task.logs.push(log_entry.clone());
                                            
                                            // 限制日志数量
                                            if task.logs.len() > 10000 {
                                                task.logs = task.logs.split_off(task.logs.len() - 10000);
                                            }
                                            
                                            // 通过事件发送日志
                                            let callbacks = callbacks_for_log.lock().unwrap();
                                            if let Some(on_progress) = callbacks.get(&task_id_for_log) {
                                                on_progress(toolset_plugin::ProgressData {
                                                    task_id: task_id_for_log.clone(),
                                                    status: "syncing".to_string(),
                                                    message: None,
                                                    progress: None,
                                                    processed_files: None,
                                                    total_files: None,
                                                    processed_size: None,
                                                    total_size: None,
                                                    log: Some(log_entry),
                                                });
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            log::info!("文件监听任务已结束");
        });

        log::info!("文件监听设置完成，准备返回");
        self.add_log(task_id, "success", "监听已启动，等待文件变化...");
        log::info!("======== start_watching 完成: task_id={} ========", task_id);
        Ok(())
    }
}

impl ToolPlugin for FolderSyncTool {
    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            id: "foldersync".to_string(),
            name: "文件夹实时同步".to_string(),
            version: "1.0.1".to_string(),
            description: "单向文件夹实时同步工具，支持多个文件夹同时监控".to_string(),
            author: "zww".to_string(),
            icon: "🔄".to_string(),
        }
    }

    fn execute(
        &self,
        method: &str,
        args: Value,
        on_progress: Box<dyn Fn(ProgressData) + Send>,
    ) -> Result<Value, String> {
        match method {
            "addSyncTask" => {
                eprintln!("[foldersync] ======== addSyncTask 开始 ========");
                eprintln!("[foldersync] 参数: {:?}", args);
                log::info!("======== addSyncTask 开始 ========");
                log::info!("参数: {:?}", args);
                let source_dir = args.get("sourceDir")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "sourceDir 参数缺失".to_string())?;
                let target_dir = args.get("targetDir")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "targetDir 参数缺失".to_string())?;

                // 验证源目录
                if !Path::new(source_dir).exists() {
                    return Err("源文件夹不存在".to_string());
                }
                if !Path::new(source_dir).is_dir() {
                    return Err("源路径不是文件夹".to_string());
                }

                // 创建目标目录
                // 尝试使用当前运行时，如果不存在则创建新的
                let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    handle.block_on(fs::create_dir_all(target_dir))
                } else {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| format!("无法创建 Tokio 运行时: {}", e))?;
                    rt.block_on(fs::create_dir_all(target_dir))
                };
                result.map_err(|e| format!("创建目标目录失败: {}", e))?;

                // 生成任务ID
                let task_id = {
                    let mut counter = self.task_counter.lock().unwrap();
                    *counter += 1;
                    format!("task_{}_{}", counter, Uuid::new_v4().simple())
                };

                let source_path = PathBuf::from(source_dir);
                let target_path = PathBuf::from(target_dir);

                // 保存 on_progress 回调以便后续发送日志事件
                {
                    eprintln!("[foldersync] 保存回调: task_id={}", task_id);
                    let mut callbacks = self.progress_callbacks.lock().unwrap();
                    callbacks.insert(task_id.clone(), on_progress);
                    eprintln!("[foldersync] 回调已保存，当前回调数量: {}", callbacks.len());
                }
                
                // 创建任务
                let task = SyncTask {
                    id: task_id.clone(),
                    source_dir: source_path.clone(),
                    target_dir: target_path.clone(),
                    status: SyncStatus::Initializing,
                    logs: Vec::new(),
                    cancelled: Arc::new(Mutex::new(false)),
                    watcher: Arc::new(Mutex::new(None)),
                };

                {
                    let mut tasks = self.tasks.lock().unwrap();
                    tasks.insert(task_id.clone(), task);
                }

                log::info!("任务已创建: task_id={}", task_id);
                self.add_log(&task_id, "info", &format!("开始初始化同步任务: {}", task_id));
                self.add_log(&task_id, "info", &format!("源文件夹: {}", source_dir));
                self.add_log(&task_id, "info", &format!("目标文件夹: {}", target_dir));

                // 初始化同步
                log::info!("准备调用 initialize_sync, task_id={}", task_id);
                // 尝试使用当前运行时，如果不存在则创建新的
                let handle_result = tokio::runtime::Handle::try_current();
                log::info!("Tokio 运行时检查: {:?}", if handle_result.is_ok() { "已存在" } else { "不存在，将创建新的" });
                
                let result = if let Ok(handle) = handle_result {
                    log::info!("使用现有运行时调用 initialize_sync");
                    handle.block_on(self.initialize_sync(&task_id))
                } else {
                    log::info!("创建新的运行时并调用 initialize_sync");
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| format!("无法创建 Tokio 运行时: {}", e))?;
                    rt.block_on(self.initialize_sync(&task_id))
                };
                log::info!("initialize_sync 调用完成，结果: {:?}", if result.is_ok() { "成功" } else { "失败" });
                match result {
                    Ok(_) => {
                        log::info!("初始化同步完成，准备启动文件监听...");
                        // 启动监听
                        match self.start_watching(&task_id) {
                            Ok(_) => {
                                log::info!("文件监听启动成功");
                                Ok(serde_json::json!({
                                    "success": true,
                                    "taskId": task_id,
                                    "message": "同步任务已启动"
                                }))
                            }
                            Err(e) => {
                                log::error!("启动文件监听失败: {}", e);
                                self.add_log(&task_id, "error", &format!("启动文件监听失败: {}", e));
                                Err(format!("启动文件监听失败: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("初始化同步失败: {}", e);
                        let mut tasks = self.tasks.lock().unwrap();
                        if let Some(task) = tasks.get_mut(&task_id) {
                            task.status = SyncStatus::Error(e.clone());
                        }
                        self.add_log(&task_id, "error", &format!("初始化失败: {}", e));
                        Err(e)
                    }
                }
            }
            "stopSyncTask" => {
                let task_id = args.get("taskId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "taskId 参数缺失".to_string())?;

                self.stop_task(task_id)?;

                Ok(serde_json::json!({
                    "success": true,
                    "message": "同步任务已停止"
                }))
            }
            "getTaskLogs" => {
                let task_id = args.get("taskId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "taskId 参数缺失".to_string())?;
                let limit = args.get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as usize;

                let logs = self.get_task_logs(task_id, limit)?;
                let logs_json: Vec<Value> = logs.into_iter().map(|log| {
                    serde_json::json!({
                        "time": log.time,
                        "type": log.level,
                        "message": log.message,
                    })
                }).collect();

                Ok(serde_json::json!({
                    "success": true,
                    "logs": logs_json,
                    "total": logs_json.len()
                }))
            }
            _ => Err(format!("未知方法: {}", method)),
        }
    }

    fn stop_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| "任务不存在".to_string())?;

        *task.cancelled.lock().unwrap() = true;
        task.status = SyncStatus::Stopped;

        // 停止watcher
        let mut watcher_guard = task.watcher.lock().unwrap();
        *watcher_guard = None;
        drop(watcher_guard);

        self.add_log(task_id, "info", "同步任务已停止");
        Ok(())
    }

    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, String> {
        let tasks = self.tasks.lock().unwrap();
        let task = tasks.get(task_id)
            .ok_or_else(|| "任务不存在".to_string())?;

        let status_str = match task.status {
            SyncStatus::Initializing => "initializing",
            SyncStatus::Syncing => "syncing",
            SyncStatus::Stopped => "stopped",
            SyncStatus::Error(_) => "error",
        }.to_string();

        Ok(TaskStatus {
            task_id: task_id.to_string(),
            status: status_str,
            message: None,
            progress: None,
        })
    }

    fn get_task_logs(&self, task_id: &str, limit: usize) -> Result<Vec<LogEntry>, String> {
        let tasks = self.tasks.lock().unwrap();
        let task = tasks.get(task_id)
            .ok_or_else(|| "任务不存在".to_string())?;

        let logs = task.logs.clone();
        let start = if logs.len() > limit {
            logs.len() - limit
        } else {
            0
        };

        Ok(logs[start..].to_vec())
    }
}

/// 创建工具插件实例
/// 这个函数会被框架动态加载
#[no_mangle]
pub extern "C" fn create_tool_plugin() -> *mut dyn ToolPlugin {
    Box::into_raw(Box::new(FolderSyncTool::new()))
}

