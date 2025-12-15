use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 用户提现地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserCashOutAddress {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: Option<i64>,
    pub user_group: Option<i32>,
    pub user_account: String,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub name: Option<String>,
    pub net_work: Option<String>,
    pub coin_id: Option<String>,
    pub address: Option<String>,
    pub deleted: bool,
    pub create_time: Option<DateTime>,
}

crud!(AppUserCashOutAddress {}, "app_user_cash_out_address");

impl AppUserCashOutAddress {
    pub const TABLE_NAME: &'static str = "app_user_cash_out_address";
}
