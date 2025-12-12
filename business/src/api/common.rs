use common::response::R;
use actix_web::{get, post, web, Responder};
use serde::Deserialize;
use common::error::AppError;

#[derive(Debug, Deserialize)]
pub struct IdReq {
    pub id: u64,
}

// 测试请求参数错误时有没有返回统一格式，例如id必填，但是没有接收到id
// error_handler::query_config()
#[get("/api/common/test_query")]
pub async fn test_query(req: web::Query<IdReq>) -> Result<impl Responder, AppError> {
    log::info!("req: {:?}", req.into_inner().id);
    R::ok()
}

// 测试请求体错误时有没有返回统一格式, 例如id必填，或者id类型为u64,但是实际接收到是string
// error_handler::json_config()
#[post("/api/common/test_body")]
pub async fn test_body(req: web::Json<IdReq>) -> Result<impl Responder, AppError> {
    log::info!("req: {:?}", req.into_inner().id);
    R::ok()
}

/// GET /api/common/i18n/test?key=error.insufficient_balance
/// 测试国际化翻译
#[get("/api/common/test")]
pub async fn test() -> Result<impl Responder, AppError> {
    let test = String::from("hello world!");
    R::success(test)
}