use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 代理线入金币种
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAgentCashInCoin {
    pub id: Option<i64>,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub enable: bool,
    pub show_name: Option<String>,
    pub coin_id: Option<String>,
    pub net_work: Option<String>,
    pub address: Option<String>,
    pub link: Option<String>,
    pub del: bool,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppAgentCashInCoin {}, "app_agent_cash_in_coin");

impl AppAgentCashInCoin {
    pub const TABLE_NAME: &'static str = "app_agent_cash_in_coin";
}
