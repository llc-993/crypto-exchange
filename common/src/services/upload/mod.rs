use async_trait::async_trait;
use crate::error::AppError;
use crate::models::vo::FileVO;
use std::collections::HashMap;
use std::sync::Arc;
use rbatis::RBatis;
use tokio::sync::RwLock;
use crate::services::upload::local::LocalUploadService;
use crate::utils::redis_util::RedisUtil;

#[cfg(feature = "with-s3")]
use crate::services::upload::s3::S3UploadService;

pub mod local;

#[cfg(feature = "with-s3")]
pub mod s3;

/// 文件上传数据
/// 
/// 包含处理后的文件数据和元信息
#[derive(Debug, Clone)]
pub struct FileData {
    /// 原始文件名
    pub file_name: String,
    /// 文件内容类型
    pub content_type: String,
    /// 文件二进制数据
    pub data: Vec<u8>,
}

/// 上传服务接口
/// 
/// 所有上传服务实现都需要实现此 trait
#[async_trait]
pub trait UploadService: Send + Sync {
    /// 存储文件
    /// 
    /// # Arguments
    /// * `file_data` - 文件数据（已从 Multipart 中提取）
    /// 
    /// # Returns
    /// * `Result<FileVO, AppError>` - 文件信息或错误
    async fn store_file(&self, file_data: FileData) -> Result<FileVO, AppError>;
}

/// 上传服务注册表
/// 
/// 管理所有已注册的上传服务实现，支持动态注册和查询
pub struct UploadServiceRegistry {
    services: Arc<RwLock<HashMap<String, Arc<dyn UploadService>>>>,
}

impl UploadServiceRegistry {
    /// 创建新的服务注册表
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册上传服务
    /// 
    /// # Arguments
    /// * `name` - 服务名称（如 "loadUpload", "awsS3UploadServiceImpl"）
    /// * `service` - 服务实现
    pub async fn register(&self, name: String, service: Arc<dyn UploadService>) {
        log::info!("注册上传服务: {}", name);
        self.services.write().await.insert(name, service);
    }
    
    /// 获取上传服务
    /// 
    /// # Arguments
    /// * `name` - 服务名称
    /// 
    /// # Returns
    /// * `Option<Arc<dyn UploadService>>` - 服务实现（如果已注册）
    pub async fn get(&self, name: &str) -> Option<Arc<dyn UploadService>> {
        self.services.read().await.get(name).cloned()
    }
    
    /// 获取所有已注册的服务名称
    pub async fn list_services(&self) -> Vec<String> {
        self.services.read().await.keys().cloned().collect()
    }
}

impl Default for UploadServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 上传服务支持（核心调度器）
/// 
/// 根据配置动态选择对应的上传服务实现
pub struct UploadServiceSupport {
    registry: Arc<UploadServiceRegistry>,
    rb: Arc<RBatis>,
    redis: Arc<RedisUtil>
}

impl UploadServiceSupport {
    /// 创建新的上传服务支持实例
    /// 
    /// # Arguments
    /// * `rb` - 数据库连接
    /// * `redis` - Redis 连接
    pub async fn new(
        rb: Arc<RBatis>,
        redis: Arc<RedisUtil>
    ) -> Self {
        log::info!("📤 初始化上传服务注册表...");
        let registry = Arc::new(UploadServiceRegistry::new());
        
        // 注册本地上传服务
        let local_upload_service = Arc::new(LocalUploadService::new(
            rb.clone(),
            redis.clone()
        ));
        registry.register("loadUpload".to_string(), local_upload_service).await;
        log::info!("✅ 本地上传服务已注册");
        
        // 注册 S3 上传服务（如果启用了 with-s3 feature）
        #[cfg(feature = "with-s3")]
        {
            let s3_upload_service = Arc::new(S3UploadService::new(
                rb.clone(),
                redis.clone()
            ));
            registry.register("awsS3UploadServiceImpl".to_string(), s3_upload_service).await;
            log::info!("✅ S3 上传服务已注册");
        }
        
        log::info!("📤 上传服务注册表已就绪");
        Self { registry, rb, redis }
    }

    
    /// 存储文件（自动选择服务）
    ///
    /// # Returns
    /// * `Result<FileVO, AppError>` - 文件信息或错误
    pub async fn store_file(
        &self,
        file_data: FileData
    ) -> Result<FileVO, AppError> {
        
        // 使用 BaseConfig 的 uploadService 字段
        // 注意：BaseConfig 在 crate::models::config_mapping::base_config::BaseConfig
        
        // 使用 ConfigService 获取配置
        use crate::services::config_service::ConfigService;
        let config_service = ConfigService::new(self.rb.clone(), self.redis.clone());
        
        let service_name = config_service.get_value_by_code(
            "uploadService", 
            Some("loadUpload")
        )
        .await?
        .unwrap_or_else(|| "loadUpload".to_string());
        
            
        log::info!("使用上传服务: {}", service_name);
        
        // 动态获取服务
        let service = self.registry.get(&service_name).await
            .ok_or_else(|| {
                let available = futures::executor::block_on(self.registry.list_services());
                log::error!(
                    "上传服务未找到: {}, 可用服务: {:?}", 
                    service_name, 
                    available
                );
                AppError::business(&format!(
                    "Upload service '{}' not found. Available services: {:?}", 
                    service_name,
                    available
                ))
            })?;
        
        service.store_file(file_data).await
    }
}
