use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 操作日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysOptLog {
    pub id: Option<i64>,
    pub opt_user: Option<String>,
    pub ip: Option<String>,
    pub remark: Option<String>,
    pub json: Option<String>,
    pub create_time: Option<DateTime>,
}

crud!(SysOptLog {}, "sys_opt_log");

impl SysOptLog {
    pub const TABLE_NAME: &'static str = "sys_opt_log";
}
