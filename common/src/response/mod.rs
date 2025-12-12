use actix_web::{HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct R<T> {
    pub code: u16,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}


impl<T: Serialize> R<T> {
    /// 成功响应，返回 Result 类型以便直接在 handler 中使用
    pub fn success(data: T) -> Result<R<T>, crate::error::AppError> {
        Ok(Self {
            code: 200,
            msg: "success".to_string(),
            data: Some(data),
        })
    }

    pub fn error(code: u16, msg: String) -> Self {
        Self {
            code,
            msg,
            data: None,
        }
    }
}

impl R<()> {
    /// 成功响应（无数据），返回 Result 类型以便直接在 handler 中使用
    pub fn ok() -> Result<R<()>, crate::error::AppError> {
        Ok(R::<()> {
            code: 200,
            msg: "success".to_string(),
            data: None,
        })
    }
}

// 为 R<T> 实现 Responder trait
impl<T: Serialize> Responder for R<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match serde_json::to_string(&self) {
            Ok(body) => HttpResponse::Ok()
                .content_type("application/json")
                .body(body),
            Err(e) => HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(format!(r#"{{"code":500,"msg":"Serialization error: {}"}}"#, e)),
        }
    }
}
