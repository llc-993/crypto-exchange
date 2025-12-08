use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 币币交易挂单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeOrder {
    pub id: Option<i64>,
    pub trade_type: i32,
    pub user_id: i64,
    pub uid: i64,
    pub user_name: String,
    pub p1_account: Option<String>,
    pub user_group: Option<i32>,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub order_number: Option<String>,
    pub symbol: Option<String>,
    pub enable_order_robot: bool,
    pub auto_trigger_time: Option<i32>,
    pub buyer_low_price_rate: Option<Decimal>,
    pub seller_over_price_rate: Option<Decimal>,
    pub order_expire_time: i32,
    pub base_symbol: Option<String>,
    pub quote_symbol: Option<String>,
    pub direction: i32,
    pub field: Option<String>,
    pub price: Option<Decimal>,
    pub transfer_price: Option<Decimal>,
    pub transfer_fee: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub actual_quantity: Option<Decimal>,
    pub unfilled_quantity: Option<Decimal>,
    pub status: i32,
    pub fee_rate: Option<Decimal>,
    pub fee: Option<Decimal>,
    pub freeze_quote: Option<Decimal>,
    pub business_type: Option<i32>,
    pub create_time: Option<DateTime>,
    pub expire_time: Option<DateTime>,
    pub settlement_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub can_show: Option<bool>,
    pub can_sell: Option<bool>,
}

crud!(AppExchangeOrder {}, "app_exchange_order");

impl AppExchangeOrder {
    pub const TABLE_NAME: &'static str = "app_exchange_order";
}
