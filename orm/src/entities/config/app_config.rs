use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// app配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub id: Option<i64>,
    pub code: Option<String>,
    pub value: Option<String>,
    pub remark: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppConfig {}, "app_config");

impl AppConfig {
    pub const TABLE_NAME: &'static str = "app_config";
}
