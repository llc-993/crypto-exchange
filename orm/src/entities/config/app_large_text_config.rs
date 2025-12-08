use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// app配置(大文本)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLargeTextConfig {
    pub id: Option<i64>,
    pub code: Option<String>,
    pub group_id: i32,
    pub sort_level: i32,
    pub i18n_code: Option<String>,
    pub link: Option<String>,
    pub title: Option<String>,
    pub value: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppLargeTextConfig {}, "app_large_text_config");

impl AppLargeTextConfig {
    pub const TABLE_NAME: &'static str = "app_large_text_config";
}
