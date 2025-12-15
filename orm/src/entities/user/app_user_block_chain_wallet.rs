use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use serde::{Deserialize, Serialize};

/// 用户区块链钱包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUserBlockChainWallet {
    pub id: Option<i64>,
    pub user_id: i64,
    pub uid: i64,
    pub network: Option<String>,
    pub address: Option<String>,
    pub params: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

crud!(AppUserBlockChainWallet {}, "app_user_block_chain_wallet");

impl AppUserBlockChainWallet {
    pub const TABLE_NAME: &'static str = "app_user_block_chain_wallet";
}
