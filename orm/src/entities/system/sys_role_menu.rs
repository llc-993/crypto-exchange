use rbatis::crud;
use serde::{Deserialize, Serialize};

/// 角色和菜单关联表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysRoleMenu {
    pub role_id: i64,
    pub menu_id: i64,
}

crud!(SysRoleMenu {}, "sys_role_menu");

impl SysRoleMenu {
    pub const TABLE_NAME: &'static str = "sys_role_menu";
}
