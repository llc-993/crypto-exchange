use rbatis::crud;
use serde::{Deserialize, Serialize};

/// 后台系统角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysRole {
    pub role_id: Option<i64>,
    pub role_code: Option<String>,
    pub role_name: Option<String>,
    pub role_status: Option<i32>,
}

crud!(SysRole {}, "sys_role");

impl SysRole {
    pub const TABLE_NAME: &'static str = "sys_role";
}
