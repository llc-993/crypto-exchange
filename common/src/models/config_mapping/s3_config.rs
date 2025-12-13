use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;
/// S3配置

#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    /// S3 存储桶名称
    #[config(code = "s3BucketName")]
    pub s3_bucket_name: Option<String>,
    /// S3 端点 (例如: "ap-east-1.amazonaws.com")
    #[config(code = "s3Endpoint")]
    pub s3_endpoint: Option<String>,
    /// S3 区域
    #[config(code = "s3Region")]
    pub s3_region: Option<String>,
    /// S3 访问密钥 ID
    #[config(code = "s3AccessKeyId")]
    pub s3_access_key_id: Option<String>,
    /// S3 访问密钥密文
    #[config(code = "s3AccessKeySecret")]
    pub s3_access_key_secret: Option<String>,
    /// S3 对象 key 前缀
    #[config(code = "s3KeyPrefix")]
    pub s3_key_prefix: Option<String>,
    /// S3 最大上传限制（MB）
    #[config(code = "s3MaxSizeMb")]
    pub s3_max_size_mb: Option<String>,

    /// 允许的文件类型（逗号分隔）
    #[config(code = "s3AllowedTypes", default = "image/,video/,application/pdf")]
    pub s3_allowed_types: Option<String>,
}