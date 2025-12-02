use crate::error::{AppError, AppResult};
use crate::services::storage::StorageService;
use crate::types::DependencyCheckResult;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

/// 依赖管理服务 - 负责管理工具的npm依赖
pub struct DependencyManager {
    storage: Arc<StorageService>,
}

impl DependencyManager {
    pub fn new(storage: Arc<StorageService>) -> Self {
        Self { storage }
    }
    
    /// 获取工具依赖目录
    pub fn get_tool_dependencies_dir(&self, tool_id: &str) -> PathBuf {
        self.storage.get_data_dir().join("tool-deps").join(tool_id)
    }
    
    /// 检查依赖是否已安装
    pub fn check_dependencies_installed(
        &self,
        tool_id: &str,
        dependencies: &HashMap<String, String>,
    ) -> AppResult<DependencyCheckResult> {
        if dependencies.is_empty() {
            return Ok(DependencyCheckResult {
                installed: true,
                missing: Vec::new(),
                details: HashMap::new(),
            });
        }
        
        let deps_dir = self.get_tool_dependencies_dir(tool_id);
        let node_modules = deps_dir.join("node_modules");
        
        if !node_modules.exists() {
            return Ok(DependencyCheckResult {
                installed: false,
                missing: dependencies.keys().cloned().collect(),
                details: dependencies
                    .keys()
                    .map(|k| (k.clone(), false))
                    .collect(),
            });
        }
        
        let mut missing = Vec::new();
        let mut details = HashMap::new();
        
        for (dep_name, _version) in dependencies.iter() {
            let dep_path = node_modules.join(dep_name);
            let installed = dep_path.exists();
            
            if !installed {
                missing.push(dep_name.clone());
            }
            
            details.insert(dep_name.clone(), installed);
        }
        
        Ok(DependencyCheckResult {
            installed: missing.is_empty(),
            missing,
            details,
        })
    }
    
    /// 安装依赖
    pub async fn install_dependencies(
        &self,
        tool_id: &str,
        dependencies: &HashMap<String, String>,
        on_progress: Option<Box<dyn Fn(String, f64) + Send>>,
    ) -> AppResult<()> {
        if dependencies.is_empty() {
            return Ok(());
        }
        
        let deps_dir = self.get_tool_dependencies_dir(tool_id);
        
        // 确保目录存在
        fs::create_dir_all(&deps_dir)?;
        
        // 创建package.json
        let package_json = serde_json::json!({
            "name": format!("tool-deps-{}", tool_id),
            "version": "1.0.0",
            "private": true,
            "dependencies": dependencies
        });
        
        let package_json_path = deps_dir.join("package.json");
        fs::write(
            &package_json_path,
            serde_json::to_string_pretty(&package_json)?,
        )?;
        
        log::info!("开始为工具 {} 安装依赖...", tool_id);
        
        if let Some(ref progress) = on_progress {
            progress("开始安装依赖...".to_string(), 0.0);
        }
        
        // 使用npm安装依赖
        let output = Command::new("npm")
            .arg("install")
            .arg("--production")
            .arg("--no-audit")
            .arg("--no-fund")
            .current_dir(&deps_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AppError::Dependency(format!("执行npm install失败: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Dependency(format!("npm install失败: {}", stderr)));
        }
        
        log::info!("✓ 工具 {} 的依赖安装完成", tool_id);
        
        if let Some(ref progress) = on_progress {
            progress("依赖安装完成".to_string(), 100.0);
        }
        
        Ok(())
    }
    
    /// 清理工具依赖
    pub fn cleanup_tool_dependencies(&self, tool_id: &str) -> AppResult<()> {
        let deps_dir = self.get_tool_dependencies_dir(tool_id);
        
        if deps_dir.exists() {
            fs::remove_dir_all(&deps_dir)?;
            log::info!("已清理工具 {} 的依赖", tool_id);
        }
        
        Ok(())
    }
    
    /// 清理所有工具依赖
    pub fn cleanup_all_dependencies(&self) -> AppResult<()> {
        let tool_deps_dir = self.storage.get_data_dir().join("tool-deps");
        
        if tool_deps_dir.exists() {
            fs::remove_dir_all(&tool_deps_dir)?;
            log::info!("已清理所有工具依赖");
        }
        
        Ok(())
    }
}

