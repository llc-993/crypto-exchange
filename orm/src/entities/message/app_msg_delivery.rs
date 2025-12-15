use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 消息投递表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMsgDelivery {
    pub id: Option<i64>,
    pub message_id: i64,
    pub user_id: i64,
    pub delivery_status: String,
    pub delivered_at: DateTime,
    pub read_at: Option<DateTime>,
    pub is_deleted: i8,
}

crud!(AppMsgDelivery {}, "app_msg_delivery");

impl AppMsgDelivery {
    pub const TABLE_NAME: &'static str = "app_msg_delivery";
}
