use rbatis::{crud, impl_select};
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 用户表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUser {
    pub id: Option<i64>,
    pub uid: i64,
    pub t1_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub t3_id: Option<i64>,
    pub p1_account: Option<String>,
    pub user_name: Option<String>,
    pub user_account: Option<String>,
    pub acc_type: Option<i32>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub mobile_phone: Option<String>,
    pub share_code: Option<String>,
    pub password: Option<String>,
    pub show_password: Option<String>,
    pub money_password: Option<String>,
    pub show_money_password: Option<String>,
    pub source_host: Option<String>,
    pub avatar: Option<String>,
    pub user_group: Option<i32>,
    pub frozen: Option<bool>,
    pub register_ip: Option<String>,
    pub register_area: Option<String>,
    pub register_time: Option<DateTime>,
    pub credit_score: Option<i32>,
    pub last_login_ip: Option<String>,
    pub last_login_time: Option<DateTime>,
    pub cert_level: i32,
    pub real_name: String,
    pub id_card_front: String,
    pub id_card_back: String,
    pub id_number: String,
    pub address: String,
    pub id_card_in_hand: String,
    pub bank_name: Option<String>,
    pub bank_branch_name: Option<String>,
    pub bank_card_number: Option<String>,
    pub bank_card_user_name: Option<String>,
    pub sec_contract_control: Option<bool>,
    pub sec_contract_rate: Option<Decimal>,
    pub buy_times: Option<i32>,
}

crud!(AppUser {}, "app_user");
impl_select!(AppUser{select_by_name(name: &str) -> Option => "`where user_account = #{name} LIMIT 1`" });
impl_select!(AppUser{select_by_id(id: i64) -> Option => "`where id = #{id} LIMIT 1`"});
impl_select!(AppUser{select_by_share_code(code: &str) -> Option => "`where share_code = #{code} LIMIT 1`"});


impl AppUser {
    pub const TABLE_NAME: &'static str = "app_user";
}
