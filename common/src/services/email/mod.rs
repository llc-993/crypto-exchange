use async_trait::async_trait;
use crate::error::AppError;
use crate::models::req::email_req::SendEmailReq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rbatis::RBatis;
use crate::utils::redis_util::RedisUtil;
use crate::services::email::gmail::GmailEmailService;

pub mod gmail;

/// 邮件服务接口
#[async_trait]
pub trait EmailService: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;

    /// 发送邮件
    async fn send(&self, req: SendEmailReq) -> Result<(), AppError>;
}

/// 邮件服务注册表
pub struct EmailServiceRegistry {
    services: Arc<RwLock<HashMap<String, Arc<dyn EmailService>>>>,
}

impl EmailServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, name: String, service: Arc<dyn EmailService>) {
        log::info!("注册邮件服务: {}", name);
        self.services.write().await.insert(name, service);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn EmailService>> {
        self.services.read().await.get(name).cloned()
    }

    pub async fn list_services(&self) -> Vec<String> {
        self.services.read().await.keys().cloned().collect()
    }
}

impl Default for EmailServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 邮件服务支持
pub struct EmailServiceSupport {
    registry: Arc<EmailServiceRegistry>,
    rb: Arc<RBatis>,
    redis: Arc<RedisUtil>,
}

impl EmailServiceSupport {
    pub async fn new(rb: Arc<RBatis>, redis: Arc<RedisUtil>) -> Self {
        log::info!("📧 初始化邮件服务注册表...");
        let registry = Arc::new(EmailServiceRegistry::new());

        let config_service = crate::services::config_service::ConfigService::new(
            rb.clone(), 
            redis.clone()
        );
        let config_service_arc = Arc::new(config_service);

        // 注册 Gmail 服务
        let gmail_service = Arc::new(GmailEmailService::new(config_service_arc.clone()));
        registry.register("gmail".to_string(), gmail_service).await;
        log::info!("✅ Gmail邮件服务已注册");

        Self { registry, rb, redis }
    }

    pub async fn send(&self, req: SendEmailReq) -> Result<(), AppError> {
        use crate::services::config_service::ConfigService;
        let config_service = ConfigService::new(self.rb.clone(), self.redis.clone());

        let service_name = config_service.get_value_by_code(
            "emailService", 
            Some("gmail")
        )
        .await?
        .unwrap_or_else(|| "gmail".to_string());

        log::info!("使用邮件服务: {}", service_name);

        let service = self.registry.get(&service_name).await
            .ok_or_else(|| {
                let available = futures::executor::block_on(self.registry.list_services());
                log::error!("邮件服务未找到: {}, 可用服务: {:?}", service_name, available);
                AppError::business(&format!(
                    "Email service '{}' not found. Available services: {:?}", 
                    service_name, 
                    available
                ))
            })?;

        service.send(req).await
    }
}
