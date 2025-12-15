use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 充值申请
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRechargeOrder {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub user_group: Option<i32>,
    pub user_account: String,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub p1_account: Option<String>,
    pub channel_id: i64,
    pub channel_name: Option<String>,
    pub channel_address: Option<String>,
    pub pay_img: Option<String>,
    pub user_hash: Option<String>,
    pub ip: String,
    pub order_no: Option<String>,
    pub apply_time: Option<DateTime>,
    pub apply_amount: Option<Decimal>,
    pub coin_id: Option<String>,
    pub net_work: Option<String>,
    pub status: Option<i32>,
    pub two_step_confirmed: Option<bool>,
    pub confirmed_hash: Option<String>,
    pub reason: Option<String>,
    pub remit_time: Option<DateTime>,
    pub hash: Option<String>,
    pub operator_user: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppRechargeOrder {}, "app_recharge_order");

impl AppRechargeOrder {
    pub const TABLE_NAME: &'static str = "app_recharge_order";
}
