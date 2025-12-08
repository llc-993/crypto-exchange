use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 认购申请记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppIeoRequest {
    pub id: Option<i64>,
    pub r#type: i32,
    pub project_id: i64,
    pub project_name: Option<String>,
    pub symbol: Option<String>,
    pub base_symbol: Option<String>,
    pub quote_symbol: Option<String>,
    pub publish_price: Decimal,
    pub user_id: i64,
    pub uid: i64,
    pub user_group: Option<i32>,
    pub user_account: String,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub p1_account: Option<String>,
    pub order_no: Option<String>,
    pub apply_time: Option<DateTime>,
    pub buy_total: i64,
    pub amount: Decimal,
    pub review_total: i64,
    pub review_amount: Decimal,
    pub status: i32,
    pub reason: Option<String>,
    pub remit_time: Option<DateTime>,
    pub pay_time: Option<DateTime>,
    pub pay_status: i32,
    pub create_time: Option<DateTime>,
}

crud!(AppIeoRequest {}, "app_ieo_request");

impl AppIeoRequest {
    pub const TABLE_NAME: &'static str = "app_ieo_request";
}
