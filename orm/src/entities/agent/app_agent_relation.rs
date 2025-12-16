use rbatis::{crud, impl_select};
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 代理层级关联表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAgentRelation {
    pub id: Option<i64>,
    pub t1_id: i64,
    pub t2_id: i64,
    pub t3_id: Option<i64>,
    pub top_share_code: String,
    pub level: i32,
    pub user_group: i32,
    pub ori_user_id: i64,
    pub ori_share_code: String,
    pub ori_account: String,
    pub p1_id: i64,
    pub p1_code: String,
    pub p1_account: String,
    pub p2_id: i64,
    pub p2_code: String,
    pub p2_account: String,
    pub p3_id: i64,
    pub p3_code: String,
    pub p3_account: String,
    pub p4_id: Option<i64>,
    pub p5_id: Option<i64>,
    pub p6_id: Option<i64>,
    pub p7_id: Option<i64>,
    pub p8_id: Option<i64>,
    pub p9_id: Option<i64>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppAgentRelation {}, "app_agent_relation");
impl_select!(AppAgentRelation{select_by_share_code(code: &str) -> Option => "`where ori_share_code = #{code} LIMIT 1`" });
impl_select!(AppAgentRelation{select_by_ori_user_id(ori_user_id: i64) -> Option => "`where ori_user_id = #{ori_user_id} LIMIT 1`"});

impl AppAgentRelation {
    pub const TABLE_NAME: &'static str = "app_agent_relation";
}
