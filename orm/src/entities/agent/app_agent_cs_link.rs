use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 客服链接列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAgentCsLink {
    pub id: Option<i64>,
    pub name: String,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub enable: bool,
    pub create_time: Option<DateTime>,
    pub link: Option<String>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppAgentCsLink {}, "app_agent_cs_link");

impl AppAgentCsLink {
    pub const TABLE_NAME: &'static str = "app_agent_cs_link";
}
