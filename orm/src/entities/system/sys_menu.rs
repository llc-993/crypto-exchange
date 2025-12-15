use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 菜单权限表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMenu {
    pub menu_id: Option<i64>,
    pub menu_name: String,
    pub parent_id: Option<i64>,
    pub order_num: Option<i32>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub query: Option<String>,
    pub menu_type: Option<i32>,
    pub status: Option<bool>,
    pub perms: Option<String>,
    pub icon: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
    pub remark: Option<String>,
}

crud!(SysMenu {}, "sys_menu");

impl SysMenu {
    pub const TABLE_NAME: &'static str = "sys_menu";
}
