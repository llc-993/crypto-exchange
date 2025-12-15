use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 新币申购
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppIeo {
    pub id: Option<i64>,
    pub project_name: Option<String>,
    pub r#type: Option<i32>,
    pub symbol: Option<String>,
    pub base_symbol: Option<String>,
    pub quote_symbol: Option<String>,
    pub publish_price: Decimal,
    pub publish_total: i64,
    pub buy_total: i64,
    pub remaining_progress: Decimal,
    pub buy_limit_max: i64,
    pub buy_limit_min: i64,
    pub buy_max_times: Option<i32>,
    pub ad_time_start: Option<DateTime>,
    pub sell_time_start: Option<DateTime>,
    pub sell_time_end: Option<DateTime>,
    pub issued_time: Option<DateTime>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub white_paper: Option<String>,
    pub is_show: bool,
    pub create_by: Option<String>,
    pub deleted: bool,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppIeo {}, "app_ieo");

impl AppIeo {
    pub const TABLE_NAME: &'static str = "app_ieo";
}
