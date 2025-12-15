use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 锁仓挖矿项目订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMineOrder {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub user_name: String,
    pub user_group: Option<i32>,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub order_number: Option<String>,
    pub project_id: i64,
    pub project_name: String,
    pub coin_symbol: Option<String>,
    pub lock_day: i32,
    pub amount: Decimal,
    pub rate_day_limit: Decimal,
    pub rate_day_max: Decimal,
    pub income_limit: Option<Decimal>,
    pub income_max: Option<Decimal>,
    pub income: Option<Decimal>,
    pub liquidated_rate: Decimal,
    pub liquidated_amount: Option<Decimal>,
    pub expired_time: Option<DateTime>,
    pub last_income_time: Option<DateTime>,
    pub income_days: i32,
    pub status: i32,
    pub reason: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppMineOrder {}, "app_mine_order");

impl AppMineOrder {
    pub const TABLE_NAME: &'static str = "app_mine_order";
}
