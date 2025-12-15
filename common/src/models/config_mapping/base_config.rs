use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;
/// 基础配置
#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct BaseConfig {
    /// 上传服务
    #[config(code = "uploadService", default = "loadUpload")]
    pub upload_service: Option<String>,
    
    /// 注册频率限制/天
    #[config(code = "regLimitDay", default = "999")]
    pub reg_limit_day: Option<i32>,
    /// 默认头像
    #[config(code = "defaultAvatar", default = "/")]
    pub default_avatar: Option<String>,

    /// 邮件服务
    #[config(code = "emailService", default = "gmail")]
    pub email_service: Option<String>,

    /// 短信服务
    #[config(code = "smsService", default = "dxb")]
    pub sms_service: Option<String>,

}