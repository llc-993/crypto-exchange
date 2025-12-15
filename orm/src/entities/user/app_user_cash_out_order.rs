use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 用户提现申请订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserCashOutOrder {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub user_group: Option<i32>,
    pub user_account: String,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub p1_account: Option<String>,
    pub ip: String,
    pub order_no: Option<String>,
    pub apply_time: Option<DateTime>,
    pub apply_amount: Option<Decimal>,
    pub actual_amount: Option<Decimal>,
    pub fee: Option<Decimal>,
    pub business_type: Option<i32>,
    pub remark: Option<String>,
    pub status: Option<i32>,
    pub reason: Option<String>,
    pub coin_id: Option<String>,
    pub user_band_address_id: i64,
    pub net_work: Option<String>,
    pub address: Option<String>,
    pub sys_wallet_id: i64,
    pub sys_address: Option<String>,
    pub sys_reason: Option<String>,
    pub tx_status: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_branch_name: Option<String>,
    pub bank_card_number: Option<String>,
    pub bank_card_user_name: Option<String>,
    pub remit_time: Option<DateTime>,
    pub hash: Option<String>,
    pub operator_user: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppUserCashOutOrder {}, "app_user_cash_out_order");

impl AppUserCashOutOrder {
    pub const TABLE_NAME: &'static str = "app_user_cash_out_order";
}
