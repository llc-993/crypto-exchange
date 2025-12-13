use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use log::{info, warn};
use rbatis::RBatis;
use crate::models::config_mapping::local_upload_config::LocalUploadConfig;
use crate::error::AppError;
use crate::models::vo::FileVO;

use crate::utils::redis_util::RedisUtil;
use super::{UploadService, FileData};
use crate::utils::snowflake;

/// 本地文件系统上传服务
/// 
/// 将文件保存到本地文件系统，支持文件类型检测、大小限制等功能
pub struct LocalUploadService {
    rb: Arc<RBatis>,
    redis: Arc<RedisUtil>
}

impl LocalUploadService {
    /// 创建本地上传服务实例

    pub fn new(
        rb: Arc<RBatis>,
        redis: Arc<RedisUtil>
    ) -> Self {
        Self { rb, redis }
    }
    
    /// 确保上传目录存在
    async fn ensure_upload_dir(&self, store_root: &String) -> Result<(), AppError> {
        if !Path::new(store_root).exists() {
            fs::create_dir_all(store_root).await?;
            info!("创建上传目录: {}", store_root);
        }
        Ok(())
    }
}

#[async_trait]
impl UploadService for LocalUploadService {
    async fn store_file(&self, file_data: FileData) -> Result<FileVO, AppError> {
        info!("本地上传服务: 收到文件上传请求");
        info!("接收到文件上传: {}", file_data.file_name);
        info!("文件类型: {}", file_data.content_type);
        
        use crate::services::config_service::ConfigService;
        
        // 从 local_upload_config 获取配置参数
        let config_service = ConfigService::new(self.rb.clone(), self.redis.clone());
        let config = config_service.load_config::<LocalUploadConfig>().await?;
        
        let store_root = config.store_root
            .ok_or_else(|| AppError::unknown("error.config_not_found"))?;
        
        let file_host = config.file_host
            .ok_or_else(|| AppError::unknown("error.config_not_found"))?;
        
        let max_size_mb = config.max_size_mb
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(10);
        
        // 解析允许的文件类型
        let allowed_types: Vec<String> = config.allowed_types
            .unwrap_or_else(|| "image/,video/,application/pdf".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // 确保上传目录存在
        self.ensure_upload_dir(&store_root).await?;

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

        // 保存文件
        let path = Path::new(&store_root).join(&unique_name);
        let mut file = fs::File::create(&path).await?;
        file.write_all(&file_data.data).await?;

        info!("文件保存成功: {}", path.to_string_lossy());

        Ok(FileVO::new(unique_name, file_host))
    }
}
