use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 秒合约订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSecondsContractOrder {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub user_name: String,
    pub user_group: Option<i32>,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub order_number: Option<String>,
    pub seconds_id: i64,
    pub expire_seconds: i32,
    pub rate_win_in_come: Option<Decimal>,
    pub free_rate: Option<Decimal>,
    pub symbol: Option<String>,
    pub coin_symbol: Option<String>,
    pub direction: i32,
    pub result_direction: i32,
    pub win_lose: bool,
    pub enable_preset: bool,
    pub preset_win_lose: Option<bool>,
    pub preset_buying_price: Option<Decimal>,
    pub preset_end_price: Option<Decimal>,
    pub preset_status: i32,
    pub status: i32,
    pub buying_price: Option<Decimal>,
    pub end_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub in_come: Option<Decimal>,
    pub free: Option<Decimal>,
    pub balance_change: Option<Decimal>,
    pub create_time: Option<DateTime>,
    pub expire_time: Option<DateTime>,
    pub settlement_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppSecondsContractOrder {}, "app_seconds_contract_order");

impl AppSecondsContractOrder {
    pub const TABLE_NAME: &'static str = "app_seconds_contract_order";
}
