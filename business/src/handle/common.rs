use common::response::R;
use actix_web::{get, post, web, Responder};
use serde::Deserialize;
use common::error::AppError;
use common::models::config_mapping::base_config::BaseConfig;
use common::services::upload::FileData;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use crate::state::AppState;

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


#[derive(Deserialize)]
pub struct IpQuery {
    pub ip: String,
    #[serde(default)]
    pub force: bool,
}

/// GET /api/common/by_ip?ip=8.8.8.8&force=false
#[get("/api/common/by_ip")]
pub async fn query_ip_address(
    query: web::Query<IpQuery>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    log::info!("收到IP归属地查询请求: ip={}, force={}", query.ip, query.force);

    let address = state.ip_service.get_real_address_by_ip(&query.ip, query.force).await?;

    match address {
        Some(addr) => {
            log::info!("IP {} 归属地: {}", query.ip, addr);
            R::success(addr)
        }
        None => {
            log::warn!("无法获取 IP {} 的归属地", query.ip);
            R::success("未知".to_string())
        }
    }
}

/// GET /api/common/config
/// 获取基础配置
#[get("/api/common/config")]
pub async fn config(
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let base_config: BaseConfig = state.config_service.load_config().await?;
    R::success(base_config)
}


/// 上传图片接口
///
/// 从配置服务读取当前使用的上传服务名称，动态选择对应的上传实现
#[post("/api/common/uploadImg")]
pub async fn upload_image(
    state: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<impl Responder, AppError> {
    log::info!("收到文件上传请求");

    // 从 Multipart 中提取文件数据
    let mut file_data_opt: Option<FileData> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let file_name = content_disposition
            .and_then(|cd| cd.get_filename())
            .unwrap_or("unknown")
            .to_string();

        let content_type = field.content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // 读取文件数据
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| {
                AppError::business_with_params(
                    "error.upload_failed",
                    serde_json::json!({"reason": e.to_string()})
                )
            })?;
            data.extend_from_slice(&chunk);
        }

        file_data_opt = Some(FileData {
            file_name,
            content_type,
            data,
        });
        break; // 只处理第一个文件
    }

    let file_data = file_data_opt.ok_or_else(|| AppError::business("error.upload_no_file"))?;

    // 通过服务支持上传文件
    let file_vo = state.upload_service.store_file(file_data).await?;

    R::success(file_vo)
}