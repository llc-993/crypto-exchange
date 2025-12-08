use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 用户信息表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysUser {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub owner: Option<String>,
    pub share_code: Option<String>,
    pub role_code: Option<String>,
    pub user_group: i32,
    pub ga_key: Option<String>,
    pub enable_safe_mode: bool,
    pub password: Option<String>,
    pub dept_id: Option<i32>,
    pub status: Option<i8>,
    pub deleted: bool,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(SysUser {}, "sys_user");

impl SysUser {
    pub const TABLE_NAME: &'static str = "sys_user";
}
