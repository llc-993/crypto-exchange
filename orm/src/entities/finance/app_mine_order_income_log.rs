use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 锁仓挖矿项目订单收益记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMineOrderIncomeLog {
    pub id: Option<i64>,
    pub user_id: i64,
    pub order_id: i64,
    pub income: Option<Decimal>,
    pub day_key: Option<String>,
    pub create_time: Option<DateTime>,
}

crud!(AppMineOrderIncomeLog {}, "app_mine_order_income_log");

impl AppMineOrderIncomeLog {
    pub const TABLE_NAME: &'static str = "app_mine_order_income_log";
}
