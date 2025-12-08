use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 代理迁移记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAgentMoveLog {
    pub id: Option<i64>,
    pub from_user_account: Option<String>,
    pub to_user_account: Option<String>,
    pub content: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
}

crud!(AppAgentMoveLog {}, "app_agent_move_log");

impl AppAgentMoveLog {
    pub const TABLE_NAME: &'static str = "app_agent_move_log";
}
