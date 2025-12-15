use actix_web::{error::JsonPayloadError, web, HttpResponse};
use crate::response::R;

/// 自定义 JSON 错误处理器
/// 
/// 处理 JSON 反序列化错误，返回统一的错误响应格式
pub fn json_error_handler(err: JsonPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
    let error_msg = match &err {
        JsonPayloadError::Deserialize(e) => {
            format!("参数格式错误: {}", e)
        }
        JsonPayloadError::ContentType => {
            "Content-Type 必须是 application/json".to_string()
        }
        JsonPayloadError::Overflow { limit } => {
            format!("请求体过大，限制为 {} 字节", limit)
        }
        _ => "JSON 解析失败".to_string(),
    };

    let response: R<()> = R {
        code: 400,
        msg: error_msg,
        data: None,
    };

    actix_web::error::InternalError::from_response(
        err,
        HttpResponse::BadRequest().json(response),
    )
    .into()
}

/// 自定义 Query 错误处理器
/// 
/// 处理 Query 参数解析错误，返回统一的错误响应格式
pub fn query_error_handler(err: actix_web::error::QueryPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
    let error_msg = format!("参数错误: {}", err);

    let response: R<()> = R {
        code: 400,
        msg: error_msg,
        data: None,
    };

    actix_web::error::InternalError::from_response(
        err,
        HttpResponse::BadRequest().json(response),
    )
    .into()
}

/// 注册 JSON 错误处理器的辅助函数
pub fn json_config() -> web::JsonConfig {
    web::JsonConfig::default().error_handler(json_error_handler)
}

/// 注册 Query 错误处理器的辅助函数
pub fn query_config() -> web::QueryConfig {
    web::QueryConfig::default().error_handler(query_error_handler)
}
