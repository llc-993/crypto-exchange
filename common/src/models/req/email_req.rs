use serde::{Deserialize, Serialize};

/// 发送邮件请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailReq {
    /// 收件人地址
    pub to_address: String,
    /// 邮件主题
    pub subject: String,
    /// 邮件正文（HTML）
    pub html_body: String,
}
