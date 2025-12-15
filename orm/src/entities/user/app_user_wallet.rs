use rbatis::crud;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 用户钱包信息表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserWallet {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub coin_id: String,
    pub sort_level: i32,
    pub user_group: i32,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub user_account: String,
    pub balance: Decimal,
    pub temp_asset: Decimal,
    pub frozen_balance: Decimal,
    pub fee: Option<Decimal>,
    pub income: Decimal,
    pub rebate: Decimal,
    pub ts: i64,
    pub cash_out: Option<Decimal>,
    pub cash_in: Option<Decimal>,
}

crud!(AppUserWallet {}, "app_user_wallet");

impl AppUserWallet {
    pub const TABLE_NAME: &'static str = "app_user_wallet";
}
