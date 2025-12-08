use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 认证申请
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCertRequest {
    pub id: Option<i64>,
    pub cert_level: i32,
    pub user_id: i64,
    pub uid: i64,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub user_name: String,
    pub real_name: String,
    pub email: String,
    pub id_card_front: String,
    pub id_card_back: String,
    pub id_number: String,
    pub address: String,
    pub id_card_in_hand: String,
    pub reject_reason: Option<String>,
    pub status: i32,
    pub opt_user: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
}

crud!(AppCertRequest {}, "app_cert_request");

impl AppCertRequest {
    pub const TABLE_NAME: &'static str = "app_cert_request";
}
