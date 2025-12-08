use rbatis::crud;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 资金费率历史记录表
/// 
/// 记录永续合约的资金费率历史数据
/// 
/// # 表名
/// `app_exchange_futures_funding_rate`
/// 
/// # 主要字段
/// - `symbol`: 交易对符号
/// - `funding_rate`: 资金费率
/// - `mark_price`: 标记价格
/// - `timestamp`: 结算时间戳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeFuturesFundingRate {
    /// 主键ID
    pub id: Option<i64>,
    /// 交易对符号，如 BTCUSDT
    pub symbol: String,
    /// 资金费率
    pub funding_rate: Decimal,
    /// 标记价格
    pub mark_price: Decimal,
    /// 结算时间戳（毫秒）
    pub timestamp: i64,
}

crud!(AppExchangeFuturesFundingRate {}, "app_exchange_futures_funding_rate");
rbatis::impl_select!(AppExchangeFuturesFundingRate {
    select_by_symbol_time(symbol: String, start_time: i64, end_time: i64) => 
    "`where symbol = #{symbol} and timestamp >= #{start_time} and timestamp <= #{end_time} order by timestamp desc`" 
}, "app_exchange_futures_funding_rate");

impl AppExchangeFuturesFundingRate {
    pub const TABLE_NAME: &'static str = "app_exchange_futures_funding_rate";
}
