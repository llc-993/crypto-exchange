use common::response::R;
use actix_web::{get, post, web, HttpRequest, Responder};
use serde::{Deserialize, Serialize};
use common::error::AppError;
use common::models::config_mapping::base_config::BaseConfig;
use common::services::upload::FileData;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use rbatis::executor::Executor;
use crate::state::AppState;
use common::models::req::app_token::{LoginRequest, RegisterRequest};
use orm::entities::user::AppUser;
use orm::entities::user::app_user_wallet::AppUserWallet;
use orm::entities::agent::AppAgentRelation;
use sa_token_plugin_actix_web::sa_token_core::StpUtil;
use common::models::dto::login::LoginResponse;
use rbatis::RBatis;
use rbatis::rbdc::datetime::DateTime;
use rbs::value;
use common::mq::message_queue::Message;
use rust_decimal::Decimal;
use common::utils::generator_id_util;

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

/// POST /api/user/auth/register
#[post("/api/user/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    payload: web::Json<RegisterRequest>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    log::info!("收到注册请求: userAccount={}", payload.user_account);

    // 1. 校验 RegisterRequest
    if payload.user_account.is_empty() {
        return Err(AppError::validation("validation.user_account_required"));
    }
    if payload.password.is_empty() {
        return Err(AppError::validation("validation.password_required"));
    }
    if payload.money_password.is_empty() {
        return Err(AppError::validation("validation.money_password_required"));
    }

    // 2. 检查用户是否已存在
    let existing_user = AppUser::select_by_name(state.rb.as_ref(), &payload.user_account)
        .await
        .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?;
    
    if existing_user.is_some() {
        return Err(AppError::validation("validation.user_already_exists"));
    }

    // 3. 获取注册 IP
    let register_ip = req.connection_info().realip_remote_addr().map(|s| s.to_string());

    // 4. 解析上级代理
    let final_agent = resolve_parent_agent(state.rb.as_ref(), &payload.invite_code).await?;

    // 5. 组装搜索关键词
    let invite_code_str = payload.invite_code.as_deref().unwrap_or("8888");
    let keyword = format!("{};{}", payload.user_account, invite_code_str);

    // 6. 开启事务并执行所有数据库操作
    let tx = state.rb.acquire_begin().await
        .map_err(|e| AppError::unknown_with_params("error.db_transaction", 
            serde_json::json!({"msg": e.to_string()})))?;

    // defer_async 如果tx丢弃将回滚
    let mut tx = tx.defer_async(|tx| async move {
        if !tx.done() {
            tx.rollback().await.unwrap();
            log::info!("rollback");
        }
    });

    // 创建用户

    let now = DateTime::now();
    
    let mut app_user = AppUser {
        id: None,
        uid: 0, // 临时值，后面会更新
        t1_id: Some(final_agent.t1_id),
        t2_id: Some(final_agent.t2_id),
        t3_id: final_agent.t3_id,
        p1_account: Some(final_agent.ori_account.clone()),
        user_name: Some(payload.user_account.clone()),
        user_account: Some(payload.user_account.clone()),
        acc_type: Some(0),
        email: None,
        country: None,
        mobile_phone: None,
        share_code: None,
        password: Some(payload.password.clone()),
        show_password: Some(payload.password.clone()),
        money_password: Some(payload.money_password.clone()),
        show_money_password: Some(payload.money_password.clone()),
        source_host: None,
        avatar: None,
        user_group: Some(0),
        frozen: Some(false),
        register_ip: register_ip.clone(),
        register_area: None,
        register_time: Some(now),
        credit_score: Some(100),
        last_login_ip: None,
        last_login_time: None,
        cert_level: 0,
        real_name: String::new(),
        id_card_front: String::new(),
        id_card_back: String::new(),
        id_number: String::new(),
        address: String::new(),
        id_card_in_hand: String::new(),
        bank_name: None,
        bank_branch_name: None,
        bank_card_number: None,
        bank_card_user_name: None,
        sec_contract_control: None,
        sec_contract_rate: None,
        buy_times: Some(0),
    };

    // 插入用户
    let row = AppUser::insert(&mut tx, &app_user).await
        .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?;

    // 获取插入后的用户ID
    let user_id = row.last_insert_id.as_i64().unwrap_or(0);
    let share_code = generator_id_util::generate_for_id(user_id);
    
    app_user.id = Some(user_id);
    app_user.share_code = Some(share_code.clone());

    // 生成 UID
    let uid = generator_id_util::generate_uid(
        user_id,
        app_user.user_group.unwrap_or(0) as i64,
        final_agent.t1_id,
        final_agent.t2_id,
    );
    app_user.uid = uid;

    // 更新用户的 share_code 和 UID
    tx.exec(
        "UPDATE app_user SET uid = ?, share_code = ? WHERE id = ?", 
        vec![rbs::to_value!(uid), rbs::to_value!(share_code), rbs::to_value!(user_id)])
        .await
        .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?;

    // 7. 创建钱包 AppUserWallet
    let wallet = AppUserWallet {
        id: None,
        user_id,
        uid,
        coin_id: "USDT".to_string(),
        sort_level: 0,
        user_group: app_user.user_group.unwrap_or(0),
        top_user_id: app_user.t1_id,
        t2_id: app_user.t2_id,
        user_account: payload.user_account.clone(),
        balance: Decimal::ZERO,
        temp_asset: Decimal::ZERO,
        frozen_balance: Decimal::ZERO,
        fee: Some(Decimal::ZERO),
        income: Decimal::ZERO,
        rebate: Decimal::ZERO,
        ts: chrono::Utc::now().timestamp(),
        cash_out: Some(Decimal::ZERO),
        cash_in: Some(Decimal::ZERO),
    };

    AppUserWallet::insert(&mut tx, &wallet).await
        .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?;

    // 8. 创建代理关系
    let agent_relation = state.agent_relation_service
        .create_app_user_agent_relation_tx(&mut tx, &app_user, Some(&final_agent))
        .await?;

    // 提交事务
    tx.commit().await
        .map_err(|e| AppError::unknown_with_params("error.db_transaction", 
            serde_json::json!({"msg": e.to_string()})))?;

    log::info!("用户注册成功: userAccount={}, userId={}", payload.user_account, user_id);

    // 9. 发布用户注册事件（异步）
    if let Some(ip) = register_ip {
        let mq = state.mq.clone();
        let username = payload.user_account.clone();
        tokio::spawn(async move {
            if let Err(e) = publish_user_registered_event(user_id as u64, username, ip, &mq).await {
                log::error!("Failed to publish user registered event: {}", e);
            }
        });
    }

    // 10. 返回登录 token
    let user_id_str = user_id.to_string();
    let token = StpUtil::login(&user_id_str)
        .await
        .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Token generation failed: {}", e)})))?;

    StpUtil::set_session_value(&user_id_str, "username", &payload.user_account)
        .await
        .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Failed to set session: {}", e)})))?;

    let response = LoginResponse {
        token: token.to_string(),
        account: payload.user_account.clone(),
        avatar: None,
    };

    R::success(response)
}

/// 解析上级代理
async fn resolve_parent_agent(
    rb: &RBatis,
    invite_code: &Option<String>,
) -> Result<AppAgentRelation, AppError> {
    let parent_agent = if let Some(code) = invite_code {
        if !code.is_empty() {
            AppAgentRelation::select_by_share_code(rb, code).await
                .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?
        } else {
            None
        }
    } else {
        None
    };

    // 如果没有找到父代理,使用根代理(ID=1)
    if parent_agent.is_none() {
        AppAgentRelation::select_by_ori_user_id(rb, 1).await
            .map_err(|e| AppError::unknown_with_params("error.database", serde_json::json!({"msg": e.to_string()})))?
            .ok_or_else(|| AppError::unknown("error.root_agent_not_found"))
    } else {
        Ok(parent_agent.unwrap())
    }
}

/// 用户注册事件载荷
#[derive(Debug, Serialize, Deserialize)]
struct UserRegisteredPayload {
    user_id: u64,
    username: String,
    ip: Option<String>,
}

/// 发布用户注册事件
async fn publish_user_registered_event(
    user_id: u64,
    username: String,
    register_ip: String,
    mq: &common::mq::message_queue::MessageQueue,
) -> Result<(), AppError> {
    
    let event_payload = UserRegisteredPayload {
        user_id,
        username,
        ip: Some(register_ip),
    };
    
    let payload_value = serde_json::to_value(&event_payload)
        .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Failed to serialize event payload: {}", e)})))?;
        
    let message = Message::new("user.registered", payload_value);
    mq.publish(&message).await
        .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Failed to publish event: {}", e)})))?;

    Ok(())
}


pub async fn get_profile() {
    // 获取用户资料逻辑
}

pub async fn update_profile() {
    // 更新用户资料逻辑
}
