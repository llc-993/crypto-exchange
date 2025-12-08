use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 短信发送记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSmsSendRecord {
    pub id: Option<i64>,
    pub env: String,
    pub platform: String,
    pub ip: String,
    pub mobile: String,
    pub content: String,
    pub status: bool,
    pub send_time: Option<DateTime>,
}

crud!(AppSmsSendRecord {}, "app_sms_send_record");

impl AppSmsSendRecord {
    pub const TABLE_NAME: &'static str = "app_sms_send_record";
}
