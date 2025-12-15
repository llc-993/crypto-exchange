use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 永续合约(伪)订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeCrossOrder {
    pub id: Option<i64>,
    pub business_type: i32,
    pub trade_type: i32,
    pub user_id: i64,
    pub uid: i64,
    pub user_name: String,
    pub user_group: Option<i32>,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub p1_account: Option<String>,
    pub order_number: Option<String>,
    pub symbol: Option<String>,
    pub lever_code: String,
    pub lever: i32,
    pub open_price: Option<Decimal>,
    pub trigger_value: Option<Decimal>,
    pub lose_value: Option<Decimal>,
    pub mark_price: Option<Decimal>,
    pub quit_price: Option<Decimal>,
    pub focus_amount: Option<Decimal>,
    pub over_price: Option<Decimal>,
    pub freeze: bool,
    pub quantity: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub balance: Option<Decimal>,
    pub balance_percentage: Option<Decimal>,
    pub fee: Option<Decimal>,
    pub cross_amount: Option<Decimal>,
    pub roe: Option<Decimal>,
    pub in_come: Option<Decimal>,
    pub status: i32,
    pub create_time: Option<DateTime>,
    pub settlement_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub ts: i64,
    pub t1_id: Option<i64>,
}

crud!(AppExchangeCrossOrder {}, "app_exchange_cross_order");

impl AppExchangeCrossOrder {
    pub const TABLE_NAME: &'static str = "app_exchange_cross_order";
}
