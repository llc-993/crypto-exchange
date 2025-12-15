use common::response::R;
use actix_web::{get, post, web, Responder};
use serde::Deserialize;
use common::error::AppError;
use common::models::config_mapping::base_config::BaseConfig;
use common::services::upload::FileData;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use crate::state::AppState;
use common::models::req::app_token::LoginRequest;
use orm::entities::app_user::AppUser;
use sa_token_plugin_actix_web::sa_token_core::StpUtil;
use common::models::dto::login::LoginResponse;

/// POST /api/user/auth/login
#[post("/api/user/auth/login")]
pub async fn login(
    state: web::Data<AppState>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    log::info!("收到登录请求: userAccount={}", payload.user_account);

    // Find user
    let user = AppUser::select_by_name(state.rb.as_ref(), &payload.user_account)
        .await
        .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?
        .ok_or_else(|| AppError::validation("validation.invalid_credentials"))?;

    // Check if frozen
    if user.frozen.unwrap_or(false) {
        return Err(AppError::validation("validation.account_frozen"));
    }

    // Verify password
    let stored_password = user.password.as_ref()
        .ok_or_else(|| AppError::validation("validation.invalid_credentials"))?;

    /*let hashed_input = format!("{:x}", md5::compute(&payload.password));
    if hashed_input != *stored_password {
        return Err(AppError::validation("validation.invalid_credentials"));
    }*/
    // kotlin版对比密码
    let input_password = payload.password.clone();
    if input_password != *stored_password {
        return Err(AppError::validation("validation.invalid_credentials"));
    }

    log::info!("User login verified: userAccount={}", payload.user_account);

    // Generate token
    let user_id = user.id.unwrap_or(0);
    let user_id_str = user_id.to_string();
    let token = StpUtil::login(&user_id_str)
        .await
        .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Token generation failed: {}", e)})))?;

    // Set session
    StpUtil::set_session_value(&user_id_str, "username", &user.user_account.clone().unwrap_or_default())
        .await
        .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Failed to set session: {}", e)})))?;

    let response = LoginResponse {
        token: token.to_string(),
        account: user.user_account.unwrap_or_default(),
        avatar: user.avatar,
    };

    log::info!("登录成功: userAccount={}", payload.user_account);
    R::success(response)
}


// 用户API模块
pub async fn register() {
    // 用户注册逻辑
}


pub async fn get_profile() {
    // 获取用户资料逻辑
}

pub async fn update_profile() {
    // 更新用户资料逻辑
}
