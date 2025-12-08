use rbatis::crud;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 指数价格记录表
/// 
/// 记录永续合约的指数价格历史数据
/// 
/// # 表名
/// `app_exchange_futures_index_price`
/// 
/// # 主要字段
/// - `symbol`: 交易对符号
/// - `price`: 指数价格
/// - `source`: 价格来源（如 Binance）
/// - `timestamp`: 记录时间戳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeFuturesIndexPrice {
    /// 主键ID
    pub id: Option<i64>,
    /// 交易对符号，如 BTCUSDT
    pub symbol: String,
    /// 指数价格
    pub price: Decimal,
    /// 价格来源，如 Binance
    pub source: Option<String>,
    /// 记录时间戳（毫秒）
    pub timestamp: i64,
}

crud!(AppExchangeFuturesIndexPrice {}, "app_exchange_futures_index_price");
rbatis::impl_select!(AppExchangeFuturesIndexPrice {
    select_by_symbol_time(symbol: String, start_time: i64, end_time: i64) => 
    "`where symbol = #{symbol} and timestamp >= #{start_time} and timestamp <= #{end_time} order by timestamp desc`" 
}, "app_exchange_futures_index_price");
rbatis::impl_select!(AppExchangeFuturesIndexPrice {
    select_latest_by_symbol(symbol: String) => 
    "`where symbol = #{symbol} order by timestamp desc limit 1`" 
}, "app_exchange_futures_index_price");

impl AppExchangeFuturesIndexPrice {
    pub const TABLE_NAME: &'static str = "app_exchange_futures_index_price";
}
