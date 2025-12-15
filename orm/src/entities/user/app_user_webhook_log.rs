use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// webhook记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserWebhookLog {
    pub id: Option<i64>,
    pub user_id: i64,
    pub coin_id: String,
    pub network: Option<String>,
    pub address: Option<String>,
    pub txid: Option<String>,
    pub ts: Option<i64>,
    pub direction: i32,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub symbol: String,
    pub token_decimal: i32,
    pub reason: Option<String>,
    pub ip: Option<String>,
    pub amount: Option<Decimal>,
    pub status: i32,
    pub settlement_time: Option<DateTime>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppUserWebhookLog {}, "app_user_webhook_log");

impl AppUserWebhookLog {
    pub const TABLE_NAME: &'static str = "app_user_webhook_log";
}
