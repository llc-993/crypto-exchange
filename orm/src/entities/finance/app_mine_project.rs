use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 锁仓挖矿项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMineProject {
    pub id: Option<i64>,
    pub name: String,
    pub coin_symbol: Option<String>,
    pub lock_day: i32,
    pub amount_min: Decimal,
    pub amount_max: Decimal,
    pub rate_day_limit: Decimal,
    pub rate_day_max: Decimal,
    pub liquidated_rate: Decimal,
    pub enable: bool,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppMineProject {}, "app_mine_project");

impl AppMineProject {
    pub const TABLE_NAME: &'static str = "app_mine_project";
}
