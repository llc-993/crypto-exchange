use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 会员账变记录表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserGoldChange {
    pub id: Option<i64>,
    pub serial_no: Option<String>,
    pub user_id: Option<i64>,
    pub uid: i64,
    pub coin_id: String,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub asset_type: Option<i32>,
    pub user_account: String,
    pub change_type: Option<i32>,
    pub before_amount: Option<Decimal>,
    pub after_amount: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub op_note: Option<String>,
    pub create_time: Option<DateTime>,
    pub user_group: Option<i32>,
    pub ts: i64,
    pub del: Option<bool>,
    pub ref_id: Option<i64>,
    pub change_type_name: Option<String>,
}

crud!(AppUserGoldChange {}, "app_user_gold_change");

impl AppUserGoldChange {
    pub const TABLE_NAME: &'static str = "app_user_gold_change";
}
