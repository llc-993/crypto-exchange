use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;

/// Gmail配置
#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct GmailConfig {
    /// SMTP服务器
    #[config(code = "smtpHost")]
    pub smtp_host: Option<String>,
    /// 用户名
    #[config(code = "gmailUsername")]
    pub gmail_username: Option<String>,
    /// 密码
    #[config(code = "gmailPassword")]
    pub gmail_password: Option<String>,
}
