use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 秒合约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSecondsContract {
    pub id: Option<i64>,
    pub expire_seconds: i32,
    pub rate_win_in_come: Option<Decimal>,
    pub enable_preset: bool,
    pub rate_win_success: Option<Decimal>,
    pub min_limit: Option<Decimal>,
    pub max_limit: Option<Decimal>,
    pub free_rate: Option<Decimal>,
    pub enable: bool,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppSecondsContract {}, "app_seconds_contract");

impl AppSecondsContract {
    pub const TABLE_NAME: &'static str = "app_seconds_contract";
}
