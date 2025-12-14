use async_trait::async_trait;
use crate::error::AppError;
use crate::models::req::sms_req::SendSmsReq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rbatis::RBatis;
use crate::utils::redis_util::RedisUtil;
use crate::services::sms::dxb::DXBSmsService;

pub mod dxb;

/// SMS服务接口
#[async_trait]
pub trait SmsService: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;

    /// 发送短信
    async fn send(&self, req: SendSmsReq) -> Result<bool, AppError>;
}

/// SMS服务注册表
pub struct SmsServiceRegistry {
    services: Arc<RwLock<HashMap<String, Arc<dyn SmsService>>>>,
}

impl SmsServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, name: String, service: Arc<dyn SmsService>) {
        log::info!("注册SMS服务: {}", name);
        self.services.write().await.insert(name, service);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn SmsService>> {
        self.services.read().await.get(name).cloned()
    }

    pub async fn list_services(&self) -> Vec<String> {
        self.services.read().await.keys().cloned().collect()
    }
}

impl Default for SmsServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// SMS服务支持
pub struct SmsServiceSupport {
    registry: Arc<SmsServiceRegistry>,
    rb: Arc<RBatis>,
    redis: Arc<RedisUtil>,
}

impl SmsServiceSupport {
    pub async fn new(rb: Arc<RBatis>, redis: Arc<RedisUtil>) -> Self {
        log::info!("📨 初始化SMS服务注册表...");
        let registry = Arc::new(SmsServiceRegistry::new());

        let config_service = crate::services::config_service::ConfigService::new(
            rb.clone(),
            redis.clone()
        );
        let config_service_arc = Arc::new(config_service);

        // 注册 短信宝 服务
        let dxb_service = Arc::new(DXBSmsService::new(config_service_arc.clone()));
        registry.register("dxb".to_string(), dxb_service).await;
        log::info!("✅ 短信宝SMS服务已注册");

        Self { registry, rb, redis }
    }

    pub async fn send(&self, req: SendSmsReq) -> Result<bool, AppError> {
        use crate::services::config_service::ConfigService;
        let config_service = ConfigService::new(self.rb.clone(), self.redis.clone());

        let service_name = config_service.get_value_by_code(
            "smsService", 
            Some("dxb")
        )
        .await?
        .unwrap_or_else(|| "dxb".to_string());

        log::info!("使用SMS服务: {}", service_name);

        let service = self.registry.get(&service_name).await
            .ok_or_else(|| {
                let available = futures::executor::block_on(self.registry.list_services());
                log::error!("SMS服务未找到: {}, 可用服务: {:?}", service_name, available);
                AppError::business(&format!(
                    "SMS service '{}' not found. Available services: {:?}", 
                    service_name, 
                    available
                ))
            })?;

        service.send(req).await
    }
}
