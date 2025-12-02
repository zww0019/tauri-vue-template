use crate::error::{AppError, AppResult};
use crate::types::{ToolInfo, ToolManifest};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// 工具信息结构
pub struct Tool {
    pub name: String,
    pub manifest: ToolManifest,
    pub path: PathBuf,
    pub entry: PathBuf,
}

/// 工具管理服务 - 负责加载和管理工具
pub struct ToolsManager {
    tools_dir: PathBuf,
    tools: Arc<Mutex<HashMap<String, Tool>>>,
}

impl ToolsManager {
    pub fn new(app_handle: &tauri::AppHandle) -> AppResult<Self> {
        // 获取资源目录中的tools路径
        let resource_path = app_handle
            .path()
            .resource_dir()
            .map_err(|e| AppError::Tool(format!("无法获取资源目录: {}", e)))?;
        
        let tools_dir = resource_path.join("tools");
        
        log::info!("工具目录: {:?}", tools_dir);
        
        // 确保工具目录存在
        if !tools_dir.exists() {
            fs::create_dir_all(&tools_dir)?;
            log::warn!("工具目录不存在，已创建: {:?}", tools_dir);
        }
        
        let mut manager = Self {
            tools_dir,
            tools: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 加载所有工具
        manager.load_tools()?;
        
        Ok(manager)
    }
    
    /// 加载所有工具
    fn load_tools(&mut self) -> AppResult<()> {
        if !self.tools_dir.exists() {
            log::warn!("工具目录不存在: {:?}", self.tools_dir);
            return Ok(());
        }
        
        let entries = fs::read_dir(&self.tools_dir)?;
        let mut loaded_count = 0;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let tool_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                match self.load_tool(&tool_name, &path) {
                    Ok(_) => {
                        loaded_count += 1;
                        log::info!("✓ 工具 {} 加载成功", tool_name);
                    }
                    Err(e) => {
                        log::error!("✗ 工具 {} 加载失败: {}", tool_name, e);
                    }
                }
            }
        }
        
        log::info!("共加载 {} 个工具", loaded_count);
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
        if manifest.id.is_empty() || manifest.name.is_empty() || manifest.main.is_empty() {
            return Err(AppError::Tool(format!(
                "工具 {} 的 manifest.json 格式无效",
                tool_name
            )));
        }
        
        let entry_path = tool_path.join(&manifest.main);
        
        if !entry_path.exists() {
            return Err(AppError::Tool(format!(
                "工具 {} 的入口文件不存在: {}",
                tool_name, manifest.main
            )));
        }
        
        let tool = Tool {
            name: tool_name.to_string(),
            manifest,
            path: tool_path.clone(),
            entry: entry_path,
        };
        
        let mut tools = self.tools.lock().unwrap();
        tools.insert(tool_name.to_string(), tool);
        
        Ok(())
    }
    
    /// 获取工具列表
    pub fn get_tools_list(&self) -> Vec<ToolInfo> {
        let tools = self.tools.lock().unwrap();
        
        tools
            .values()
            .map(|tool| ToolInfo {
                id: tool.manifest.id.clone(),
                name: tool.manifest.name.clone(),
                version: tool.manifest.version.clone(),
                description: tool.manifest.description.clone().unwrap_or_default(),
                author: tool.manifest.author.clone().unwrap_or_default(),
                icon: tool.manifest.icon.clone().unwrap_or_default(),
                required_auth: tool.manifest.required_auth,
                has_dependencies: !tool.manifest.dependencies.is_empty(),
                hidden: tool.manifest.hidden,
            })
            .filter(|info| !info.hidden)
            .collect()
    }
    
    /// 获取工具
    pub fn get_tool(&self, tool_name: &str) -> Option<Tool> {
        let tools = self.tools.lock().unwrap();
        tools.get(tool_name).map(|t| Tool {
            name: t.name.clone(),
            manifest: t.manifest.clone(),
            path: t.path.clone(),
            entry: t.entry.clone(),
        })
    }
    
    /// 通过ID获取工具
    pub fn get_tool_by_id(&self, tool_id: &str) -> Option<Tool> {
        let tools = self.tools.lock().unwrap();
        
        tools.values()
            .find(|t| t.manifest.id == tool_id)
            .map(|t| Tool {
                name: t.name.clone(),
                manifest: t.manifest.clone(),
                path: t.path.clone(),
                entry: t.entry.clone(),
            })
    }
    
    /// 获取工具信息（通过ID或名称）
    pub fn get_tool_info(&self, tool_id_or_name: &str) -> Option<ToolInfo> {
        let tool = self.get_tool(tool_id_or_name)
            .or_else(|| self.get_tool_by_id(tool_id_or_name))?;
        
        Some(ToolInfo {
            id: tool.manifest.id.clone(),
            name: tool.manifest.name.clone(),
            version: tool.manifest.version.clone(),
            description: tool.manifest.description.clone().unwrap_or_default(),
            author: tool.manifest.author.clone().unwrap_or_default(),
            icon: tool.manifest.icon.clone().unwrap_or_default(),
            required_auth: tool.manifest.required_auth,
            has_dependencies: !tool.manifest.dependencies.is_empty(),
            hidden: tool.manifest.hidden,
        })
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
    pub fn get_tools_dir(&self) -> &PathBuf {
        &self.tools_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 注意：这些测试需要实际的Tauri应用环境才能运行
    // 在实际项目中，可以创建模拟的测试环境
}

