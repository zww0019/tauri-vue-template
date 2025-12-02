use crate::error::{AppError, AppResult};
use crate::tool_plugin::CreateToolPluginFn;
use toolset_plugin::ToolPlugin;
use libloading::Library;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(target_os = "macos")]
const PLUGIN_EXT: &str = "dylib";
#[cfg(target_os = "windows")]
const PLUGIN_EXT: &str = "dll";
#[cfg(target_os = "linux")]
const PLUGIN_EXT: &str = "so";

/// 工具加载器 - 负责动态加载Rust工具插件
pub struct ToolLoader;

impl ToolLoader {
    /// 加载工具插件
    /// 
    /// # Safety
    /// 此函数是 unsafe 的，因为涉及动态库加载和函数指针调用
    /// 
    /// # Arguments
    /// * `plugin_path` - 插件库文件路径
    /// 
    /// # Returns
    /// 加载的工具插件实例
    pub unsafe fn load_plugin(plugin_path: &PathBuf) -> AppResult<Arc<dyn ToolPlugin>> {
        // 验证文件存在
        if !plugin_path.exists() {
            return Err(AppError::Tool(format!(
                "插件文件不存在: {:?}",
                plugin_path
            )));
        }

        // 验证文件扩展名
        if plugin_path
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ext != PLUGIN_EXT)
            .unwrap_or(true)
        {
            return Err(AppError::Tool(format!(
                "无效的插件文件扩展名，期望: {}",
                PLUGIN_EXT
            )));
        }

        // 加载动态库
        let lib = Library::new(plugin_path)
            .map_err(|e| AppError::Tool(format!("加载插件库失败: {}", e)))?;

        // 获取创建函数
        let create_fn: libloading::Symbol<CreateToolPluginFn> = lib
            .get(b"create_tool_plugin")
            .map_err(|e| AppError::Tool(format!("找不到 create_tool_plugin 函数: {}", e)))?;

        // 创建插件实例
        let plugin = create_fn();
        if plugin.is_null() {
            return Err(AppError::Tool("插件创建函数返回空指针".to_string()));
        }

        // 包装为Arc
        // 注意：这里假设插件已经正确实现了Send + Sync
        let plugin_boxed = Box::from_raw(plugin);
        let plugin_arc = Arc::from(plugin_boxed);

        // 注意：lib需要保持加载状态，否则符号会失效
        // 这里我们使用一个技巧：将Library存储在静态或全局变量中
        // 但为了简化，我们假设插件在整个应用生命周期内保持加载
        // 在实际生产环境中，应该使用更复杂的内存管理策略
        
        // 暂时忽略lib的drop，因为我们需要保持库加载
        std::mem::forget(lib);

        Ok(plugin_arc)
    }

    /// 检查插件文件是否存在
    pub fn check_plugin_exists(tool_path: &PathBuf) -> bool {
        let plugin_name = format!("libtool_{}.{}", 
            tool_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            PLUGIN_EXT
        );
        let plugin_path = tool_path.join(&plugin_name);
        plugin_path.exists()
    }

    /// 获取插件文件路径
    pub fn get_plugin_path(tool_path: &PathBuf, tool_id: &str) -> PathBuf {
        let plugin_name = format!("libtool_{}.{}", tool_id, PLUGIN_EXT);
        tool_path.join(&plugin_name)
    }
}

