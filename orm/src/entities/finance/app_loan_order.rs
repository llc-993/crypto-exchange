use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 贷款订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLoanOrder {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub user_name: String,
    pub coin_symbol: Option<String>,
    pub order_number: Option<String>,
    pub auto_repay: bool,
    pub amount: Decimal,
    pub loan_day: i32,
    pub interest_rate_day: Decimal,
    pub interest: Option<Decimal>,
    pub request_time: Option<DateTime>,
    pub loan_time: Option<DateTime>,
    pub expired_time: Option<DateTime>,
    pub pay_time: Option<DateTime>,
    pub last_interest_time: Option<DateTime>,
    pub interest_days: i32,
    pub status: i32,
    pub reason: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppLoanOrder {}, "app_loan_order");

impl AppLoanOrder {
    pub const TABLE_NAME: &'static str = "app_loan_order";
}
