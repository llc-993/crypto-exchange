pub mod user_registered;


use common::mq::register_subscriber;
use actix_web::web;
use crate::state::AppState;
use crate::subscribers::user_registered::UserRegisteredSubscriber;

/// 注册所有订阅者
pub async fn init_subscribers(state: web::Data<AppState>) {
    log::info!("📋 Initializing message queue subscribers...");

    // 注册各个订阅者
    // 注入依赖到 UserRegisteredSubscriber
    register_subscriber(&state.mq, UserRegisteredSubscriber {
        rb: state.rb.clone(),
        ip_service: state.ip_service.clone(),
    }).await;

    log::info!("✅ All message queue subscribers initialized successfully (4 subscribers)");
}
