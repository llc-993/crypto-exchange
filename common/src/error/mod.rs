use thiserror::Error;
use serde_json::Value;
use crate::utils::i18n_helper::translate_with_params; // Removed cfg guard

pub type AppResult<T> = Result<T, AppError>;


/// 应用错误类型
///
/// 错误存储翻译键和参数，实际翻译在使用时进行
#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Validation: {key}")]
    ValidationError { key: String, params: Option<Value> },

    #[error("Auth: {key}")]
    AuthError { key: String, params: Option<Value> },

    #[error("Business: {key}")]
    BusinessError { key: String, params: Option<Value> },

    #[error("Database: {key}")]
    DatabaseError { key: String, params: Option<Value> },

    #[error("Unknown: {key}")]
    Unknown { key: String, params: Option<Value> },
}

impl AppError {
    // ===== 便捷构造方法（无参数版本）=====

    pub fn validation(key: impl Into<String>) -> Self {
        Self::ValidationError {
            key: key.into(),
            params: None
        }
    }

    pub fn auth(key: impl Into<String>) -> Self {
        Self::AuthError {
            key: key.into(),
            params: None
        }
    }

    pub fn business(key: impl Into<String>) -> Self {
        Self::BusinessError {
            key: key.into(),
            params: None
        }
    }

    pub fn database(key: impl Into<String>) -> Self {
        Self::DatabaseError {
            key: key.into(),
            params: None
        }
    }

    pub fn unknown(key: impl Into<String>) -> Self {
        Self::Unknown {
            key: key.into(),
            params: None
        }
    }

    // ===== 便捷构造方法（带参数版本）=====

    pub fn validation_with_params(key: impl Into<String>, params: Value) -> Self {
        Self::ValidationError {
            key: key.into(),
            params: Some(params)
        }
    }

    pub fn business_with_params(key: impl Into<String>, params: Value) -> Self {
        Self::BusinessError {
            key: key.into(),
            params: Some(params)
        }
    }

    pub fn database_with_params(key: impl Into<String>, params: Value) -> Self {
        Self::DatabaseError {
            key: key.into(),
            params: Some(params)
        }
    }

    /// 创建一个包含详细信息的数据库错误
    pub fn database_error(msg: impl Into<String>) -> Self {
        Self::DatabaseError {
            key: "error.database_error".to_string(),
            params: Some(serde_json::json!({"msg": msg.into()}))
        }
    }

    pub fn unknown_with_params(key: impl Into<String>, params: Value) -> Self {
        Self::Unknown {
            key: key.into(),
            params: Some(params)
        }
    }

    // ===== 获取错误信息的方法 =====

    pub fn key(&self) -> &str {
        match self {
            AppError::ValidationError { key, .. } => key,
            AppError::AuthError { key, .. } => key,
            AppError::BusinessError { key, .. } => key,
            AppError::DatabaseError { key, .. } => key,
            AppError::Unknown { key, .. } => key,
        }
    }

    pub fn params(&self) -> Option<&Value> {
        match self {
            AppError::ValidationError { params, .. } => params.as_ref(),
            AppError::AuthError { params, .. } => params.as_ref(),
            AppError::BusinessError { params, .. } => params.as_ref(),
            AppError::DatabaseError { params, .. } => params.as_ref(),
            AppError::Unknown { params, .. } => params.as_ref(),
        }
    }
}

// 实现 From trait 以支持 ? 操作符
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Unknown {
            key: "error.io_error".to_string(),
            params: Some(serde_json::json!({"msg": err.to_string()}))
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unknown {
            key: "error.unknown_error".to_string(),
            params: Some(serde_json::json!({"msg": err.to_string()}))
        }
    }
}

use actix_web::{http::StatusCode, HttpResponse, ResponseError}; // Removed cfg guard

// Removed cfg guard
impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::AuthError { .. } => StatusCode::UNAUTHORIZED,
            AppError::BusinessError { .. } => StatusCode::OK,
            AppError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unknown { .. } => StatusCode::OK,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // 网络状态码
        let server_status = self.status_code();
        // 业务状态码和网络状态码分离
        let status = match self {
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::AuthError { .. } => StatusCode::UNAUTHORIZED,
            AppError::BusinessError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unknown { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Translate the error message using the key and params
        let message = match self {
            AppError::ValidationError { key, params } => {
                log::warn!("验证错误: {}", key);
                translate_with_params(key, params.clone())
            },
            AppError::AuthError { key, params: _params } => {
                log::warn!("认证错误: {}", key);
                // For auth errors, we always return a generic "auth_failed" message
                translate_with_params("error.auth_failed", None)
            },
            AppError::BusinessError { key, params } => {
                log::error!("业务异常: {}", key);
                translate_with_params(key, params.clone())
            },
            AppError::DatabaseError { key, params } => {
                log::error!("数据库异常: {} {:?}", key, params);
                translate_with_params("error.database_error", None)
            },
            AppError::Unknown { key, params } => {
                log::error!("未知错误: {} {:?}", key, params);
                // For unknown errors, return a generic error message
                translate_with_params("error.internal_error", None)
            },
        };

        // We need to define a standard response structure here since we can't depend on business crate
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            code: i32,
            msg: String,
            data: Option<()>,
        }

        let error_response = ErrorResponse {
            code: status.as_u16() as i32,
            msg: message,
            data: None,
        };

        HttpResponse::build(server_status)
            .content_type("application/json")
            .json(error_response)
    }
}
