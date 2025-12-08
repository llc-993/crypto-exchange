use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 业绩报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysAgentReport {
    pub id: Option<i64>,
    pub date_str: String,
    pub agent_id: Option<i64>,
    pub agent_name: Option<String>,
    pub owner_id: Option<i64>,
    pub owner_name: Option<String>,
    pub level: i32,
    pub share_code: String,
    pub coin_id: String,
    pub cash_in_total: Decimal,
    pub cash_out_total: Decimal,
    pub create_time: Option<DateTime>,
}

crud!(SysAgentReport {}, "sys_agent_report");

impl SysAgentReport {
    pub const TABLE_NAME: &'static str = "sys_agent_report";
}
