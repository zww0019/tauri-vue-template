//! 工具集插件接口定义
//! 
//! 所有工具插件必须实现 `ToolPlugin` trait

use serde_json::Value;

/// 工具插件接口 - 所有工具必须实现此接口
pub trait ToolPlugin: Send + Sync {
    /// 获取工具信息
    fn get_info(&self) -> ToolInfo;

    /// 执行工具方法
    /// 
    /// # Arguments
    /// * `method` - 要执行的方法名
    /// * `args` - 方法参数（JSON格式）
    /// * `on_progress` - 进度回调函数
    /// 
    /// # Returns
    /// 执行结果（JSON格式）
    fn execute(
        &self,
        method: &str,
        args: Value,
        on_progress: Box<dyn Fn(ProgressData) + Send>,
    ) -> Result<Value, String>;

    /// 停止正在执行的任务
    /// 
    /// # Arguments
    /// * `task_id` - 任务ID
    fn stop_task(&self, task_id: &str) -> Result<(), String>;

    /// 获取任务状态
    /// 
    /// # Arguments
    /// * `task_id` - 任务ID
    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, String>;

    /// 获取任务日志
    /// 
    /// # Arguments
    /// * `task_id` - 任务ID
    /// * `limit` - 日志条数限制
    fn get_task_logs(&self, task_id: &str, limit: usize) -> Result<Vec<LogEntry>, String>;
}

/// 工具信息
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: String,
}

/// 进度数据
#[derive(Debug, Clone)]
pub struct ProgressData {
    pub task_id: String,
    pub status: String,
    pub message: Option<String>,
    pub progress: Option<f64>,
    pub processed_files: Option<usize>,
    pub total_files: Option<usize>,
    pub processed_size: Option<u64>,
    pub total_size: Option<u64>,
    pub log: Option<LogEntry>, // 可选的日志条目，用于实时日志推送
}

/// 任务状态
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub task_id: String,
    pub status: String, // "running", "completed", "failed", "stopped"
    pub message: Option<String>,
    pub progress: Option<f64>,
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: String,
    pub level: String, // "info", "success", "error", "warn", "sync"
    pub message: String,
}

/// 工具插件注册函数类型
/// 每个工具库必须导出一个名为 `create_tool_plugin` 的函数
#[allow(improper_ctypes_definitions)]
pub type CreateToolPluginFn = unsafe extern "C" fn() -> *mut dyn ToolPlugin;

/// 工具插件卸载函数类型（可选）
#[allow(improper_ctypes_definitions)]
pub type DestroyToolPluginFn = unsafe extern "C" fn(*mut dyn ToolPlugin);

