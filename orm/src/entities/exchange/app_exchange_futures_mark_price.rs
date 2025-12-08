use rbatis::crud;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 标记价格记录表
/// 
/// 记录永续合约的标记价格历史数据
/// 
/// # 表名
/// `app_exchange_futures_mark_price`
/// 
/// # 主要字段
/// - `symbol`: 交易对符号
/// - `mark_price`: 标记价格
/// - `index_price`: 指数价格
/// - `funding_basis`: 资金费率基差
/// - `timestamp`: 记录时间戳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeFuturesMarkPrice {
    /// 主键ID
    pub id: Option<i64>,
    /// 交易对符号，如 BTCUSDT
    pub symbol: String,
    /// 标记价格
    pub mark_price: Decimal,
    /// 指数价格
    pub index_price: Decimal,
    /// 资金费率基差
    pub funding_basis: Option<Decimal>,
    /// 记录时间戳（毫秒）
    pub timestamp: i64,
}

crud!(AppExchangeFuturesMarkPrice {}, "app_exchange_futures_mark_price");
rbatis::impl_select!(AppExchangeFuturesMarkPrice {
    select_by_symbol_time(symbol: String, start_time: i64, end_time: i64) => 
    "`where symbol = #{symbol} and timestamp >= #{start_time} and timestamp <= #{end_time} order by timestamp desc`" 
}, "app_exchange_futures_mark_price");
rbatis::impl_select!(AppExchangeFuturesMarkPrice {
    select_latest_by_symbol(symbol: String) => 
    "`where symbol = #{symbol} order by timestamp desc limit 1`" 
}, "app_exchange_futures_mark_price");

impl AppExchangeFuturesMarkPrice {
    pub const TABLE_NAME: &'static str = "app_exchange_futures_mark_price";
}
