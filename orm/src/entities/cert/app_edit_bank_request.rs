use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 用户修改银行卡地址请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEditBankRequest {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub user_name: Option<String>,
    pub bank_name: Option<String>,
    pub bank_branch_name: Option<String>,
    pub bank_card_number: Option<String>,
    pub bank_card_user_name: Option<String>,
    pub old_bank_name: Option<String>,
    pub old_bank_branch_name: Option<String>,
    pub old_bank_card_number: Option<String>,
    pub old_bank_card_user_name: Option<String>,
    pub last_login_ip: Option<String>,
    pub reject_reason: Option<String>,
    pub status: i32,
    pub opt_user: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppEditBankRequest {}, "app_edit_bank_request");

impl AppEditBankRequest {
    pub const TABLE_NAME: &'static str = "app_edit_bank_request";
}
