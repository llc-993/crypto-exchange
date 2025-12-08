// 错误处理模块
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("Redis错误: {0}")]
    RedisError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("未授权: {0}")]
    Unauthorized(String),

    #[error("禁止访问: {0}")]
    Forbidden(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("业务错误: {0}")]
    BusinessError(String),

    #[error("内部服务器错误: {0}")]
    InternalServerError(String),
}

pub type AppResult<T> = Result<T, AppError>;

// 从 rbatis 错误转换 (rbatis::Error 包含了 rbdc::Error)
impl From<rbatis::Error> for AppError {
    fn from(err: rbatis::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

// 从 redis 错误转换
impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::RedisError(err.to_string())
    }
}