use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 等级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLevel {
    pub id: Option<i64>,
    pub level_name: Option<String>,
    pub level_weights: i32,
    pub source_host: Option<String>,
    pub logo: Option<String>,
    pub chat_times: i32,
    pub max_task: i32,
    pub commission_rate: Option<Decimal>,
    pub content: Option<String>,
    pub enable: bool,
    pub create_by: Option<String>,
    pub create_at: i64,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppLevel {}, "app_level");

impl AppLevel {
    pub const TABLE_NAME: &'static str = "app_level";
}
