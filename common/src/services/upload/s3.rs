use std::sync::Arc;
use async_trait::async_trait;
use log::{info, warn};
use rbatis::RBatis;
use crate::error::AppError;
use crate::models::vo::FileVO;

use super::{UploadService, FileData};

use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use crate::utils::redis_util::RedisUtil;
use crate::utils::snowflake;

/// AWS S3 上传服务
/// 
/// 将文件上传到 AWS S3 存储桶

pub struct S3UploadService {
    rb: Arc<RBatis>,
    redis: Arc<RedisUtil>
}


impl S3UploadService {
    /// 创建 S3 上传服务实例
    pub fn new(
        rb: Arc<RBatis>,
        redis: Arc<RedisUtil>
    ) -> Self {
        Self { rb, redis }
    }
    
}

#[async_trait]
impl UploadService for S3UploadService {
    async fn store_file(&self, file_data: FileData) -> Result<FileVO, AppError> {
        info!("S3 上传服务: 收到文件上传请求");
        info!("接收到文件上传: {}", file_data.file_name);
        info!("文件类型: {}", file_data.content_type);
        
        // 从 base_config 获取配置参数
        use crate::models::config_mapping::s3_config::S3Config;
        use crate::services::config_service::ConfigService;
        
        let config_service = ConfigService::new(self.rb.clone(), self.redis.clone());
        let config = config_service.load_config::<S3Config>().await?;
        
        // 使用 S3 专用的最大大小限制，如果没有配置则使用全局限制
        let max_size_mb = config.s3_max_size_mb
            .and_then(|s| s.parse::<u64>().ok())
            //.or_else(|| base_config.max_size_mb.and_then(|s| s.parse::<u64>().ok())) // S3 config now separate, maybe just use default or duplicate logic if BaseConfig not available easily
            .unwrap_or(10);
        
        // 解析允许的文件类型
        let allowed_types: Vec<String> = config.s3_allowed_types
            .unwrap_or_else(|| "image/,video/,application/pdf".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let bucket = config.s3_bucket_name
            .ok_or_else(|| AppError::unknown("error.s3_bucket_not_configured"))?;
        
        // 获取 endpoint（如果没有 endpoint，需要构建标准的 AWS S3 endpoint）
        let endpoint = config.s3_endpoint.clone();
        
        // 优先使用 s3_region，如果没有则从 s3_endpoint 提取 region
        let region = if let Some(region) = config.s3_region {
            region
        } else if let Some(ref ep) = endpoint {
            // 从 endpoint 提取 region (例如: "ap-east-1.amazonaws.com" -> "ap-east-1")
            if let Some(dot_pos) = ep.find('.') {
                ep[..dot_pos].to_string()
            } else {
                return Err(AppError::unknown("error.invalid_s3_endpoint"));
            }
        } else {
            return Err(AppError::unknown("error.s3_region_not_configured"));
        };
        
        let key_prefix = config.s3_key_prefix;
        
        // 初始化 S3 客户端
        let s3_config = if let (Some(access_key_id), Some(access_key_secret)) = 
            (config.s3_access_key_id, config.s3_access_key_secret) {
            // 使用提供的凭证
            use aws_sdk_s3::config::{Credentials, Region};
            let creds = Credentials::new(
                access_key_id,
                access_key_secret,
                None,
                None,
                "static"
            );
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(Region::new(region.clone()))
                .credentials_provider(creds)
                .load()
                .await
        } else {
            // 使用默认凭证链（环境变量、IAM 角色等）
            use aws_sdk_s3::config::Region;
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(Region::new(region.clone()))
                .load()
                .await
        };
        
        let client = S3Client::new(&s3_config);
        
        // 检查文件大小
        let size_mb = file_data.data.len() as u64 / 1024 / 1024;
        if size_mb > max_size_mb {
            warn!("文件大小超过限制: {}MB > {}MB", size_mb, max_size_mb);
            return Err(AppError::business_with_params(
                "error.file_too_large", 
                serde_json::json!({"size": max_size_mb})
            ));
        }

        // 检查文件类型
        let kind = infer::get(&file_data.data);
        let mime_type = match kind {
            Some(k) => k.mime_type(),
            None => &file_data.content_type,
        };
        
        info!("检测到文件类型: {}", mime_type);

        if !allowed_types.is_empty() {
            let is_allowed = allowed_types.iter().any(|t| mime_type.starts_with(t));
            if !is_allowed {
                warn!("文件类型不允许: {}", mime_type);
                return Err(AppError::business_with_params(
                    "error.unsupported_file_type",
                    serde_json::json!({"type": mime_type})
                ));
            }
        }

        // 生成唯一文件名
        let uuid = snowflake::generate_id_string();
        let safe_name: String = file_data.file_name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect();
        let unique_name = format!("{}_{}", uuid, safe_name);

        // 构建 S3 对象 key
        let s3_key = if let Some(prefix) = &key_prefix {
            format!("{}{}", prefix, unique_name)
        } else {
            unique_name.clone()
        };

        info!("上传到 S3: bucket={}, key={}", bucket, s3_key);

        // 上传到 S3
        let byte_stream = ByteStream::from(file_data.data);
        
        match client
            .put_object()
            .bucket(&bucket)
            .key(&s3_key)
            .body(byte_stream)
            .content_type(mime_type)
            .send()
            .await
        {
            Ok(_) => {
                info!("文件上传到 S3 成功: {}", s3_key);
                // 构建 S3 URL，格式: https://{bucket}.s3.{endpoint}/
                // 匹配 Kotlin 版本: "https://${cfg.s3BucketName}.s3.${cfg.s3Endpoint}/"
                let file_host = if let Some(ep) = endpoint {
                    format!("https://{}.s3.{}/", bucket, ep)
                } else {
                    // 如果没有 endpoint，使用标准 AWS S3 格式
                    format!("https://{}.s3.{}.amazonaws.com/", bucket, region)
                };
                Ok(FileVO::new(s3_key, file_host))
            }
            Err(e) => {
                warn!("S3 上传失败: {:?}", e);
                Err(AppError::business_with_params(
                    "error.s3_upload_failed",
                    serde_json::json!({"reason": e.to_string()})
                ))
            }
        }
    }
}
