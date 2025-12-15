use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 币币交易撮合记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeMatchRecord {
    pub id: Option<i64>,
    pub symbol: Option<String>,
    pub base_symbol: Option<String>,
    pub quote_symbol: Option<String>,
    pub direction: i32,
    pub trade_type: i32,
    pub buyer_order_id: i64,
    pub seller_order_id: i64,
    pub buyer_id: i64,
    pub buyer_name: String,
    pub seller_id: i64,
    pub seller_name: String,
    pub buyer_user_group: Option<i32>,
    pub buyer_top_user_id: Option<i64>,
    pub seller_user_group: Option<i32>,
    pub seller_top_user_id: Option<i64>,
    pub buyer_price: Option<Decimal>,
    pub seller_price: Option<Decimal>,
    pub quote_amount: Option<Decimal>,
    pub base_amount: Option<Decimal>,
    pub matched: Option<Decimal>,
    pub unfreeze_quote: Option<Decimal>,
    pub buyer_fee_rate: Option<Decimal>,
    pub seller_fee_rate: Option<Decimal>,
    pub buyer_fee: Option<Decimal>,
    pub seller_fee: Option<Decimal>,
    pub status: i32,
    pub op_note: Option<String>,
    pub business_type: Option<i32>,
    pub rebate_note: Option<String>,
    pub create_time: Option<DateTime>,
    pub settlement_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppExchangeMatchRecord {}, "app_exchange_match_record");

impl AppExchangeMatchRecord {
    pub const TABLE_NAME: &'static str = "app_exchange_match_record";
}
