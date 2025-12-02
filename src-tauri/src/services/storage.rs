use crate::error::{AppError, AppResult};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 存储服务 - 负责本地数据持久化
pub struct StorageService {
    data_dir: PathBuf,
    data: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl StorageService {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("com", "zww", "ZToolSet")
            .ok_or_else(|| AppError::Storage("无法获取应用数据目录".to_string()))?;
        
        let data_dir = project_dirs.data_dir().to_path_buf();
        
        // 确保数据目录存在
        fs::create_dir_all(&data_dir)?;
        
        let mut service = Self {
            data_dir,
            data: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 加载所有存储的数据
        service.load_all()?;
        
        Ok(service)
    }
    
    /// 获取数据目录路径
    pub fn get_data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
    
    /// 获取存储文件路径
    fn get_storage_file(&self) -> PathBuf {
        self.data_dir.join("storage.json")
    }
    
    /// 加载所有数据
    fn load_all(&mut self) -> AppResult<()> {
        let storage_file = self.get_storage_file();
        
        if storage_file.exists() {
            let content = fs::read_to_string(&storage_file)?;
            let loaded_data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
            
            let mut data = self.data.lock().unwrap();
            *data = loaded_data;
            
            log::info!("已从存储文件加载 {} 条数据", data.len());
        } else {
            log::info!("存储文件不存在，使用空数据");
        }
        
        Ok(())
    }
    
    /// 保存所有数据到文件
    fn save_all(&self) -> AppResult<()> {
        let storage_file = self.get_storage_file();
        let data = self.data.lock().unwrap();
        
        let content = serde_json::to_string_pretty(&*data)?;
        fs::write(&storage_file, content)?;
        
        log::debug!("已保存 {} 条数据到存储文件", data.len());
        Ok(())
    }
    
    /// 获取值
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let data = self.data.lock().unwrap();
        
        data.get(key).and_then(|value| {
            serde_json::from_value(value.clone()).ok()
        })
    }
    
    /// 设置值
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> AppResult<()> {
        let json_value = serde_json::to_value(value)?;
        
        {
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_string(), json_value);
        }
        
        self.save_all()?;
        Ok(())
    }
    
    /// 移除值
    pub fn remove(&self, key: &str) -> AppResult<()> {
        {
            let mut data = self.data.lock().unwrap();
            data.remove(key);
        }
        
        self.save_all()?;
        Ok(())
    }
    
    /// 检查键是否存在
    pub fn has(&self, key: &str) -> bool {
        let data = self.data.lock().unwrap();
        data.contains_key(key)
    }
    
    /// 清空所有数据
    pub fn clear(&self) -> AppResult<()> {
        {
            let mut data = self.data.lock().unwrap();
            data.clear();
        }
        
        self.save_all()?;
        Ok(())
    }
    
    /// 获取所有键
    pub fn keys(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_service() {
        let storage = StorageService::new().unwrap();
        
        // 测试设置和获取
        storage.set("test_key", &"test_value".to_string()).unwrap();
        let value: Option<String> = storage.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));
        
        // 测试删除
        storage.remove("test_key").unwrap();
        let value: Option<String> = storage.get("test_key");
        assert_eq!(value, None);
    }
}

