use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;
/// 本地上传配置
#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct LocalUploadConfig {
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
}