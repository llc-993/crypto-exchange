use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// IP白名单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppWhiteIps {
    pub id: Option<i64>,
    pub name: String,
    pub rule: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppWhiteIps {}, "app_white_ips");

impl AppWhiteIps {
    pub const TABLE_NAME: &'static str = "app_white_ips";
}
