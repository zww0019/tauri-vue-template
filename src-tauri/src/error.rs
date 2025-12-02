use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("HTTP请求错误: {0}")]
    Reqwest(#[from] reqwest::Error),
    
    #[error("RSA加密错误: {0}")]
    Rsa(#[from] rsa::Error),
    
    #[error("Base64解码错误: {0}")]
    Base64(#[from] base64::DecodeError),
    
    #[error("认证错误: {0}")]
    Auth(String),
    
    #[error("工具错误: {0}")]
    Tool(String),
    
    #[error("存储错误: {0}")]
    Storage(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("无效参数: {0}")]
    InvalidArgument(String),
    
    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// API响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "操作成功".to_string(),
            data: Some(data),
        }
    }
    
    pub fn success_with_message(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }
    
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn simple_success() -> Self {
        Self {
            success: true,
            message: "操作成功".to_string(),
            data: None,
        }
    }
}

