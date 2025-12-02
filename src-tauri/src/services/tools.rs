use crate::error::{AppError, AppResult};
use crate::services::tool_loader::ToolLoader;
use crate::types::{ToolInfo, ToolManifest};
use toolset_plugin::ToolPlugin;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg(target_os = "macos")]
const PLUGIN_EXT: &str = "dylib";
#[cfg(target_os = "windows")]
const PLUGIN_EXT: &str = "dll";
#[cfg(target_os = "linux")]
const PLUGIN_EXT: &str = "so";

/// 工具信息结构
pub struct Tool {
    pub name: String,
    pub manifest: ToolManifest,
    pub path: PathBuf,
    pub plugin_path: PathBuf,
    pub plugin: Option<Arc<dyn ToolPlugin>>,
}

/// 工具管理服务 - 负责加载和管理工具
pub struct ToolsManager {
    tools_dirs: Vec<PathBuf>,  // 多个工具目录（取并集）
    tools: Arc<Mutex<HashMap<String, Tool>>>,
}

impl ToolsManager {
    pub fn new(app_handle: &tauri::AppHandle) -> AppResult<Self> {
        // 收集所有工具目录（取并集）
        let mut tools_dirs = Vec::new();
        
        if cfg!(debug_assertions) {
            // 开发环境：收集所有可能的工具目录
            
            // 1. 项目根目录的 tools 目录
            // 从可执行文件路径向上查找，直到找到包含实际工具的项目根目录
            let exe_path = std::env::current_exe()
                .map_err(|e| AppError::Tool(format!("无法获取可执行文件路径: {}", e)))?;
            let mut project_root = exe_path.clone();
            let project_tools = loop {
                let tools_path = project_root.join("tools");
                if tools_path.exists() {
                    // 检查目录中是否有实际的工具（有子目录）
                    if let Ok(mut entries) = fs::read_dir(&tools_path) {
                        let has_tools = entries.any(|entry| {
                            entry.map(|e| e.path().is_dir()).unwrap_or(false)
                        });
                        if has_tools {
                            break Some(tools_path);
                        }
                    }
                }
                if let Some(parent) = project_root.parent() {
                    project_root = parent.to_path_buf();
                } else {
                    break None;
                }
            };
            if let Some(dir) = project_tools {
                log::info!("发现项目根目录工具目录: {:?}", dir);
                tools_dirs.push(dir);
            }
            
            // 2. 资源目录（Tauri 会将 tools 复制到 target/debug/_up_/tools/）
            if let Ok(resource_path) = app_handle.path().resource_dir() {
                let resource_tools = resource_path.join("tools");
                if resource_tools.exists() {
                    log::info!("发现资源目录工具目录: {:?}", resource_tools);
                    tools_dirs.push(resource_tools);
                }
            }
            
            // 3. 用户数据目录
            if let Ok(user_data_dir) = app_handle.path().app_data_dir() {
                let user_data_tools = user_data_dir.join("tools");
                if user_data_tools.exists() {
                    log::info!("发现用户数据目录工具目录: {:?}", user_data_tools);
                    tools_dirs.push(user_data_tools);
                }
            }
        } else {
            // 打包环境：只使用 userData
            let user_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Tool(format!("无法获取用户数据目录: {}", e)))?;
            let user_data_tools = user_data_dir.join("tools");
            tools_dirs.push(user_data_tools);
        }
        
        // 如果没有找到任何工具目录，至少确保 userData 目录存在
        if tools_dirs.is_empty() {
            let user_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Tool(format!("无法获取用户数据目录: {}", e)))?;
            let user_data_tools = user_data_dir.join("tools");
            if !user_data_tools.exists() {
                fs::create_dir_all(&user_data_tools)?;
            }
            tools_dirs.push(user_data_tools);
        }
        
        log::info!("工具目录列表（共 {} 个）: {:?}", tools_dirs.len(), tools_dirs);
        
        let mut manager = Self {
            tools_dirs,
            tools: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 加载所有工具
        manager.load_tools()?;
        
        Ok(manager)
    }
    
    /// 加载所有工具（从多个目录取并集）
    fn load_tools(&mut self) -> AppResult<()> {
        let mut loaded_count = 0;
        let mut loaded_tool_ids = std::collections::HashSet::new();
        
        // 按优先级顺序扫描目录（前面的目录优先级更高）
        for tools_dir in &self.tools_dirs {
            if !tools_dir.exists() {
                log::warn!("工具目录不存在，跳过: {:?}", tools_dir);
                continue;
            }
            
            log::info!("扫描工具目录: {:?}", tools_dir);
            
            let entries = match fs::read_dir(tools_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    log::warn!("无法读取工具目录 {:?}: {}", tools_dir, e);
                    continue;
                }
            };
            
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        log::warn!("读取目录条目失败: {}", e);
                        continue;
                    }
                };
                
                let path = entry.path();
                
                if path.is_dir() {
                    let tool_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    // 先尝试加载 manifest 获取工具 ID
                    let manifest_path = path.join("manifest.json");
                    if !manifest_path.exists() {
                        log::warn!("工具目录 {} 缺少 manifest.json，跳过", tool_name);
                        continue;
                    }
                    
                    let tool_id = match fs::read_to_string(&manifest_path) {
                        Ok(content) => {
                            match serde_json::from_str::<serde_json::Value>(&content) {
                                Ok(json) => {
                                    json.get("id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| tool_name.clone())
                                }
                                Err(e) => {
                                    log::warn!("解析 manifest.json 失败: {}", e);
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("读取 manifest.json 失败: {}", e);
                            continue;
                        }
                    };
                    
                    // 如果工具已加载（从更高优先级的目录），跳过
                    if loaded_tool_ids.contains(&tool_id) {
                        log::info!("工具 {} (id: {}) 已从其他目录加载，跳过: {:?}", tool_name, tool_id, path);
                        continue;
                    }
                    
                    match self.load_tool(&tool_name, &path) {
                        Ok(_) => {
                            loaded_count += 1;
                            loaded_tool_ids.insert(tool_id);
                            log::info!("✓ 工具 {} 加载成功 (来自: {:?})", tool_name, tools_dir);
                        }
                        Err(e) => {
                            log::error!("✗ 工具 {} 加载失败: {}", tool_name, e);
                        }
                    }
                }
            }
        }
        
        log::info!("共加载 {} 个工具（从 {} 个目录）", loaded_count, self.tools_dirs.len());
        Ok(())
    }
    
    /// 加载单个工具
    fn load_tool(&self, tool_name: &str, tool_path: &PathBuf) -> AppResult<()> {
        let manifest_path = tool_path.join("manifest.json");
        
        if !manifest_path.exists() {
            return Err(AppError::Tool(format!(
                "工具 {} 缺少 manifest.json",
                tool_name
            )));
        }
        
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: ToolManifest = serde_json::from_str(&manifest_content)?;
        
        // 验证manifest
        if manifest.id.is_empty() || manifest.name.is_empty() {
            return Err(AppError::Tool(format!(
                "工具 {} 的 manifest.json 格式无效",
                tool_name
            )));
        }
        
        // 确定插件库名称
        let library_name = manifest.library.clone().unwrap_or_else(|| {
            format!("libtool_{}", manifest.id)
        });
        
        // 构建插件路径
        let plugin_path = tool_path.join(format!("{}.{}", library_name, PLUGIN_EXT));
        
        if !plugin_path.exists() {
            log::warn!(
                "工具 {} 的插件库不存在: {:?}，将在运行时尝试加载",
                tool_name, plugin_path
            );
        }
        
        // 尝试加载插件（可选，如果不存在则延迟加载）
        let plugin = if plugin_path.exists() {
            unsafe {
                match ToolLoader::load_plugin(&plugin_path) {
                    Ok(p) => {
                        log::info!("✓ 工具 {} 插件加载成功", tool_name);
                        Some(p)
                    }
                    Err(e) => {
                        log::warn!("工具 {} 插件加载失败: {}，将在运行时重试", tool_name, e);
                        None
                    }
                }
            }
        } else {
            None
        };
        
        let tool = Tool {
            name: tool_name.to_string(),
            manifest,
            path: tool_path.clone(),
            plugin_path,
            plugin,
        };
        
        let mut tools = self.tools.lock().unwrap();
        tools.insert(tool_name.to_string(), tool);
        
        Ok(())
    }
    
    /// 获取工具列表
    pub fn get_tools_list(&self) -> Vec<ToolInfo> {
        let tools = self.tools.lock().unwrap();
        log::info!("get_tools_list: 内部工具数量: {}", tools.len());
        
        let all_tools: Vec<ToolInfo> = tools
            .values()
            .map(|tool| ToolInfo {
                id: tool.manifest.id.clone(),
                name: tool.manifest.name.clone(),
                version: tool.manifest.version.clone(),
                description: tool.manifest.description.clone().unwrap_or_default(),
                author: tool.manifest.author.clone().unwrap_or_default(),
                icon: tool.manifest.icon.clone().unwrap_or_default(),
                required_auth: tool.manifest.required_auth,
                hidden: tool.manifest.hidden,
            })
            .collect();
        
        log::info!("get_tools_list: 转换后工具数量: {} (过滤前)", all_tools.len());
        
        let filtered: Vec<ToolInfo> = all_tools
            .into_iter()
            .filter(|info| {
                let keep = !info.hidden;
                if !keep {
                    log::info!("过滤掉隐藏工具: {} (id: {})", info.name, info.id);
                }
                keep
            })
            .collect();
        
        log::info!("get_tools_list: 最终返回工具数量: {}", filtered.len());
        filtered
    }
    
    /// 获取工具
    pub fn get_tool(&self, tool_name: &str) -> Option<Arc<Tool>> {
        let tools = self.tools.lock().unwrap();
        tools.get(tool_name).map(|t| Arc::new(Tool {
            name: t.name.clone(),
            manifest: t.manifest.clone(),
            path: t.path.clone(),
            plugin_path: t.plugin_path.clone(),
            plugin: t.plugin.clone(),
        }))
    }
    
    /// 通过ID获取工具
    pub fn get_tool_by_id(&self, tool_id: &str) -> Option<Arc<Tool>> {
        let tools = self.tools.lock().unwrap();
        
        tools.values()
            .find(|t| t.manifest.id == tool_id)
            .map(|t| Arc::new(Tool {
                name: t.name.clone(),
                manifest: t.manifest.clone(),
                path: t.path.clone(),
                plugin_path: t.plugin_path.clone(),
                plugin: t.plugin.clone(),
            }))
    }
    
    /// 获取工具信息（通过ID或名称）
    pub fn get_tool_info(&self, tool_id_or_name: &str) -> Option<ToolInfo> {
        let tools = self.tools.lock().unwrap();
        let tool = tools.get(tool_id_or_name)
            .or_else(|| tools.values().find(|t| t.manifest.id == tool_id_or_name))?;
        
        Some(ToolInfo {
            id: tool.manifest.id.clone(),
            name: tool.manifest.name.clone(),
            version: tool.manifest.version.clone(),
            description: tool.manifest.description.clone().unwrap_or_default(),
            author: tool.manifest.author.clone().unwrap_or_default(),
            icon: tool.manifest.icon.clone().unwrap_or_default(),
            required_auth: tool.manifest.required_auth,
            hidden: tool.manifest.hidden,
        })
    }
    
    /// 获取工具的插件实例（如果已加载）
    pub fn get_tool_plugin(&self, tool_id: &str) -> Option<Arc<dyn ToolPlugin>> {
        let tools = self.tools.lock().unwrap();
        tools.values()
            .find(|t| t.manifest.id == tool_id)
            .and_then(|t| t.plugin.clone())
    }
    
    /// 设置工具的插件实例（用于运行时加载）
    pub fn set_tool_plugin(&self, tool_id: &str, plugin: Arc<dyn ToolPlugin>) -> Result<(), String> {
        let mut tools = self.tools.lock().unwrap();
        if let Some(tool) = tools.get_mut(tool_id) {
            tool.plugin = Some(plugin);
            Ok(())
        } else {
            // 尝试通过ID查找
            let tool_name = tools.values()
                .find(|t| t.manifest.id == tool_id)
                .map(|t| t.name.clone());
            
            if let Some(name) = tool_name {
                if let Some(tool) = tools.get_mut(&name) {
                    tool.plugin = Some(plugin);
                    Ok(())
                } else {
                    Err(format!("工具不存在: {}", tool_id))
                }
            } else {
                Err(format!("工具不存在: {}", tool_id))
            }
        }
    }
    
    /// 重新加载工具
    pub fn reload_tools(&mut self) -> AppResult<()> {
        {
            let mut tools = self.tools.lock().unwrap();
            tools.clear();
        }
        
        self.load_tools()
    }
    
    /// 获取工具目录
    pub fn get_tools_dir(&self) -> Option<&PathBuf> {
        // 返回第一个工具目录（向后兼容）
        self.tools_dirs.first()
    }
    
    pub fn get_tools_dirs(&self) -> &[PathBuf] {
        &self.tools_dirs
    }
}

#[cfg(test)]
mod tests {
    // 注意：这些测试需要实际的Tauri应用环境才能运行
    // 在实际项目中，可以创建模拟的测试环境
}

