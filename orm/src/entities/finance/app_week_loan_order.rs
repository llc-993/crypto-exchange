use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 周贷款订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppWeekLoanOrder {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub user_name: String,
    pub user_group: Option<i32>,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub p1_account: String,
    pub coin_symbol: Option<String>,
    pub order_number: Option<String>,
    pub real_name: Option<String>,
    pub phone: Option<String>,
    pub ssn: Option<String>,
    pub forfeit: Option<Decimal>,
    pub holding_id_photo: Option<String>,
    pub amount: Decimal,
    pub loan_day: i32,
    pub interest_rate: Decimal,
    pub interest: Option<Decimal>,
    pub request_time: Option<DateTime>,
    pub loan_time: Option<DateTime>,
    pub expired_time: Option<DateTime>,
    pub pay_time: Option<DateTime>,
    pub repay_end_time: Option<DateTime>,
    pub should_repay_amount: Option<Decimal>,
    pub repay_amount: Option<Decimal>,
    pub status: i32,
    pub reason: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppWeekLoanOrder {}, "app_week_loan_order");

impl AppWeekLoanOrder {
    pub const TABLE_NAME: &'static str = "app_week_loan_order";
}
