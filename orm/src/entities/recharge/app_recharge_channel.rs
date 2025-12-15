use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 充值渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRechargeChannel {
    pub id: Option<i64>,
    pub show_name: Option<String>,
    pub coin_id: Option<String>,
    pub net_work: Option<String>,
    pub r#type: Option<i32>,
    pub address: Option<String>,
    pub link: Option<String>,
    pub is_show: bool,
    pub recharge_limit_min: Decimal,
    pub cash_out_limit_min: Decimal,
    pub can_recharge: bool,
    pub can_cash_out: bool,
    pub create_by: Option<String>,
    pub deleted: bool,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppRechargeChannel {}, "app_recharge_channel");

impl AppRechargeChannel {
    pub const TABLE_NAME: &'static str = "app_recharge_channel";
}
