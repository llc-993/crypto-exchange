use common::error::AppError;
use common::mq::message_queue::Message;
use common::mq::subscriber_trait::MessageSubscriber;

use common::services::ip_service::IpService;
use rbatis::RBatis;
use rbatis::executor::Executor;
use std::sync::Arc;
use async_trait::async_trait;
use common::models::req::payloads::UserRegisteredPayload;

/// 用户注册订阅者
#[derive(Clone)]
pub struct UserRegisteredSubscriber {
    pub rb: Arc<RBatis>,
    pub ip_service: Arc<IpService>,
}

#[async_trait]
impl MessageSubscriber for UserRegisteredSubscriber {
    fn topic(&self) -> &str {
        "user.registered"
    }
    
    async fn handle(&self, message: Message) -> Result<(), AppError> {
        log::info!("👤 [用户注册] 开始异步处理新用户: {:?}", message.payload);
        
        // 1. 解析参数
        let payload: UserRegisteredPayload = serde_json::from_value(message.payload)
            .map_err(|e| AppError::unknown_with_params("error.internal_error", serde_json::json!({"msg": format!("Failed to deserialize payload: {}", e)})))?;
            
        let user_id = payload.user_id;
        let ip = payload.ip;

        // 2. 查询 IP 归属地
        let mut register_area = None;
        if let Some(ref ip_addr) = ip {
            match self.ip_service.get_real_address_by_ip(&ip_addr, false).await {
                Ok(Some(addr)) => {
                    log::info!("   🌍 IP归属地: {} -> {}", ip_addr, addr);
                    register_area = Some(addr);
                }
                Ok(None) => log::warn!("   ⚠️ 无法获取IP归属地: {}", ip_addr),
                Err(e) => log::error!("   ❌ IP查询失败: {}", e),
            }
        }

        // 3. 更新用户数据 (仅更新归属地)
        if let Some(area) = register_area {
            let sql = "UPDATE app_user SET register_area = ? WHERE id = ?";
            let args = vec![
                rbs::value!(area),
                rbs::value!(user_id),
            ];

            Executor::exec(self.rb.as_ref(), sql, args).await
                .map_err(|e| AppError::unknown_with_params("error.database_error", serde_json::json!({"msg": format!("Failed to update user register_area: {}", e)})))?;
            
            log::info!("   ✅ 用户 {} 归属地更新完成", user_id);
        } else {
            log::info!("   ✅ 用户 {} 无归属地更新", user_id);
        }
        
        Ok(())
    }
}
