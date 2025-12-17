use crate::error::AppError;
use rbatis::rbdc::DateTime;
use rust_decimal::Decimal;
use crate::enums::gold_change::GoldChangeType;
use rbatis::{crud, impl_select};
use serde::{Deserialize, Serialize};

/// 会员账变记录表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserGoldChange {
    pub id: Option<i64>,
    pub serial_no: Option<String>,
    pub user_id: Option<i64>,
    pub uid: i64,
    pub coin_id: String,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub asset_type: Option<i32>,
    pub user_account: String,
    pub change_type: Option<i32>,
    pub before_amount: Option<Decimal>,
    pub after_amount: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub op_note: Option<String>,
    pub create_time: Option<DateTime>,
    pub user_group: Option<i32>,
    pub ts: i64,
    pub del: Option<bool>,
    pub ref_id: Option<i64>,
    pub change_type_name: Option<String>,
}

crud!(AppUserGoldChange {}, "app_user_gold_change");

/// 用户钱包信息表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserWallet {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub coin_id: String,
    pub sort_level: i32,
    pub user_group: i32,
    pub top_user_id: Option<i64>,
    pub t2_id: Option<i64>,
    pub user_account: String,
    pub balance: Decimal,
    pub temp_asset: Decimal,
    pub frozen_balance: Decimal,
    pub fee: Option<Decimal>,
    pub income: Decimal,
    pub rebate: Decimal,
    pub ts: i64,
    pub cash_out: Option<Decimal>,
    pub cash_in: Option<Decimal>,
}

crud!(AppUserWallet {}, "app_user_wallet");
impl_select!(AppUserWallet{select_by_user_id(user_id: i64, coin_id: String) -> Option => "`where user_id = #{user_id} and coin_id = #{coin_id} limit 1`"});


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
impl_select!(AppUser{select_by_id(id: i64) -> Option => "`where id = #{id} LIMIT 1`"});




#[derive(Debug, Clone)]
pub struct ChangeReq {
    // 必填字段（在new方法中已经设置）
    pub user_id: i64,
    pub coin_id: String,
    pub change_type: GoldChangeType,

    // 可选字段（默认None）
    pub balance: Option<Decimal>,
    pub frozen_balance: Option<Decimal>,
    pub fee: Option<Decimal>,
    pub income: Option<Decimal>,
    pub cash_in: Option<Decimal>,
    pub cash_out: Option<Decimal>,
    pub rebate: Option<Decimal>,
    pub remark: Option<String>,
    pub ref_id: Option<i64>, // 这个字段现在用不上，未来可能会有
}

impl ChangeReq {
    /// 创建新的ChangeReq实例，接收所有必填参数
    pub fn new(
        user_id: i64,
        coin_id: String,
        change_type: GoldChangeType,
    ) -> Self {
        Self {
            user_id,
            coin_id,
            change_type,
            balance: None,
            frozen_balance: None,
            fee: None,
            income: None,
            cash_in: None,
            cash_out: None,
            rebate: None,
            remark: None,
            ref_id: None,
        }
    }

    /// 设置可用余额 - 可选字段
    pub fn balance(mut self, balance: Decimal) -> Self {
        self.balance = Some(balance);
        self
    }

    /// 设置冻结余额 - 可选字段
    pub fn frozen_balance(mut self, frozen_balance: Decimal) -> Self {
        self.frozen_balance = Some(frozen_balance);
        self
    }

    /// 设置佣金 - 可选字段
    pub fn rebate(mut self, rebate: Decimal) -> Self {
        self.rebate = Some(rebate);
        self
    }

    /// 设置收入 - 可选字段
    pub fn income(mut self, income: Decimal) -> Self {
        self.income = Some(income);
        self
    }

    /// 设置备注 - 可选字段
    pub fn remark(mut self, remark: impl Into<String>) -> Self {
        self.remark = Some(remark.into());
        self
    }
    
    pub fn fee(mut self, fee: Decimal) -> Self {
        self.fee = Some(fee);
        self
    }

    /// 设置现金收入 - 可选字段
    pub fn cash_in(mut self, cash_in: Decimal) -> Self {
        self.cash_in = Some(cash_in);
        self
    }
    
    pub fn cash_out(mut self, cash_out: Decimal) -> Self {
        self.cash_out = Some(cash_out);
        self   
    }

    /// 设置引用ID - 可选字段
    pub fn ref_id(mut self, ref_id: i64) -> Self {
        self.ref_id = Some(ref_id);
        self
    }
}

pub struct WalletService;

impl WalletService {
    /// 余额变动（在事务中执行）
    /// 
    /// # Arguments
    /// * `tx` - RBatis 事务执行器
    /// * `req` - 变动请求 (ChangeReq 结构体)
    pub async fn balance_change(
        tx: &mut rbatis::executor::RBatisTxExecutorGuard,
        req: ChangeReq,
    ) -> Result<AppUserWallet, AppError> {
        let user_id = req.user_id;
        let coin_id = req.coin_id;
        let change_type = req.change_type;
        let amount = req.balance.unwrap_or(Decimal::ZERO);
        let frozen_amount = req.frozen_balance.unwrap_or(Decimal::ZERO);
        let remark = req.remark.unwrap_or_default();

        // 2. 查询钱包
        let wallet_opt = AppUserWallet::select_by_user_id(tx, user_id, coin_id.clone()).await
            .map_err(|e| {
                let err_msg = e.to_string();
                AppError::unknown_with_params("error.db_query", serde_json::json!({"msg": err_msg}))
            })?;

        let mut wallet = wallet_opt.ok_or(AppError::validation("validation.user_not_found"))?;

        // 记录变更前金额
        let before_amount = wallet.balance;

        // 3. 更新钱包
        if wallet.balance + amount < Decimal::ZERO {
            return Err(AppError::business("error.insufficient_balance"));
        }
        wallet.balance += amount;

        let before_frozen_amount = wallet.frozen_balance;
        if wallet.frozen_balance + frozen_amount < Decimal::ZERO {
            return Err(AppError::business("error.insufficient_frozen_balance"));
        }

        wallet.frozen_balance += frozen_amount;

        if let Some(income) = req.income {
            wallet.income += income;
        }
        if let Some(fee) = req.fee {
            *wallet.fee.get_or_insert(Decimal::ZERO) += fee;
        }
        if let Some(rebate) = req.rebate {
            wallet.rebate += rebate;
        }
        if let Some(cash_in) = req.cash_in {
            *wallet.cash_in.get_or_insert(Decimal::ZERO) += cash_in;
        }
        if let Some(cash_out) = req.cash_out {
            *wallet.cash_out.get_or_insert(Decimal::ZERO) += cash_out;
        }
        // 使用 update_by_map 更新
        let where_map = rbs::value!{ "user_id": user_id };
        AppUserWallet::update_by_map(tx, &wallet, where_map).await
            .map_err(|e| AppError::unknown_with_params("error.db_update", serde_json::json!({"msg": e.to_string()})))?;

        // 4. 查询用户信息用于账变记录
        let app_user = AppUser::select_by_id(tx, user_id)
            .await
            .map_err(|e| AppError::unknown_with_params("error.db_query", serde_json::json!({"msg": e.to_string()})))?
            .ok_or(AppError::validation("validation.user_not_found"))?;

        // 5. 插入账变记录
        if amount != Decimal::ZERO {
            let gold_change = AppUserGoldChange {
                id: None,
                serial_no: Some(crate::utils::snowflake::generate_id_string()),
                user_id: Some(user_id as i64),
                uid: app_user.uid,
                coin_id: coin_id.clone(),
                top_user_id: app_user.t1_id,
                t2_id: app_user.t2_id,
                asset_type: Some(0), // 0: 余额
                user_account: app_user.user_account.clone().unwrap_or(String::from("")),
                change_type: Some(change_type.get_code()),
                before_amount: Some(before_amount),
                after_amount: Some(wallet.balance),
                amount: Some(amount),
                op_note: Some(remark.clone()),
                create_time: Some(DateTime::now()),
                user_group: app_user.user_group,
                ts: -1,
                del: Some(false),
                ref_id: req.ref_id,
                change_type_name: Some(change_type.description())
            };
            AppUserGoldChange::insert(tx, &gold_change).await
                .map_err(|e| AppError::unknown_with_params("error.db_save", serde_json::json!({"msg": e.to_string()})))?;
        }

        if frozen_amount != Decimal::ZERO {
            let gold_change = AppUserGoldChange {
                id: None,
                serial_no: Some(crate::utils::snowflake::generate_id_string()),
                user_id: Some(user_id as i64),
                uid: app_user.uid,
                coin_id: coin_id.clone(),
                top_user_id: app_user.t1_id,
                t2_id: app_user.t2_id,
                asset_type: Some(1), // 1: 冻结余额
                user_account: app_user.user_account.clone().unwrap_or(String::from("")),
                change_type: Some(change_type.get_code()),
                before_amount: Some(before_frozen_amount),
                after_amount: Some(wallet.frozen_balance),
                amount: Some(frozen_amount),
                op_note: Some(remark.clone()),
                create_time: Some(DateTime::now()),
                user_group: app_user.user_group,
                ts: -1,
                del: Some(false),
                ref_id: req.ref_id,
                change_type_name: Some(change_type.description())
            };
            AppUserGoldChange::insert(tx, &gold_change).await
                .map_err(|e| AppError::unknown_with_params("error.db_save", serde_json::json!({"msg": e.to_string()})))?;
        }

        Ok(wallet)
    }
}
