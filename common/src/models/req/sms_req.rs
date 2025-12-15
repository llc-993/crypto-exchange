use serde::{Deserialize, Serialize};

/// 发送短信请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendSmsReq {
    /// 手机号
    pub mobile: String,
    /// 短信内容
    pub content: String,
}
