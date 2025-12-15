use rbatis::crud;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 贷款配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppWeekLoanSetting {
    pub id: Option<i64>,
    pub max_amount: Decimal,
    pub min_amount: Decimal,
    pub min_credit_score: i32,
    pub rate_json: String,
}

crud!(AppWeekLoanSetting {}, "app_week_loan_setting");

impl AppWeekLoanSetting {
    pub const TABLE_NAME: &'static str = "app_week_loan_setting";
}
