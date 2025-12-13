use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;
/// 基础配置
#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct BaseConfig {
    /// 上传服务
    #[config(code = "uploadService", default = "loadUpload")]
    pub upload_service: Option<String>,
    /// 文件访问域名
    #[config(code = "fileHost", default = "/")]
    pub file_host: Option<String>,
    /// 存储到服务器的路径
    #[config(code = "storeRoot", default = "./uploads")]
    pub store_root: Option<String>,
    /// 文件最大大小（MB）
    #[config(code = "maxSizeMb", default = "10")]
    pub max_size_mb: Option<String>,
    /// 允许的文件类型（逗号分隔）
    #[config(code = "allowedTypes", default = "image/,video/,application/pdf")]
    pub allowed_types: Option<String>,
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
    /// 注册频率限制/天
    #[config(code = "regLimitDay", default = "999")]
    pub reg_limit_day: Option<i32>,
    /// 默认头像
    #[config(code = "defaultAvatar", default = "/")]
    pub default_avatar: Option<String>,
    /// 商品详情跳转前缀
    #[config(code = "jumpLinkPrefix", default = "/")]
    pub jump_link_prefix: Option<String>,
}