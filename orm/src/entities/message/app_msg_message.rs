use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 消息主体表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMsgMessage {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub r#type: String,
    pub priority: i8,
    pub status: String,
    pub extra: Option<serde_json::Value>,
    pub created_by: i64,
    pub sent_by: Option<i64>,
    pub created_at: DateTime,
    pub sent_at: Option<DateTime>,
    pub expired_at: Option<DateTime>,
    pub is_deleted: i8,
}

crud!(AppMsgMessage {}, "app_msg_message");

impl AppMsgMessage {
    pub const TABLE_NAME: &'static str = "app_msg_message";
}
