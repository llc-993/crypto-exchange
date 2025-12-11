//

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinThumb {
    // 交易对
    pub symbol: String,

    // 交易资产图标（扩展字段）
    pub icon: String,

    // 开盘价
    pub open: Decimal,

    // 最高价
    pub high: Decimal,

    // 最低价
    pub low: Decimal,

    // 收盘价
    pub close: Decimal,

    // 涨跌幅
    pub chg: Decimal,

    // 涨跌额
    pub change: Decimal,

    // 交易量
    pub volume: Decimal,

    // 成交额
    pub quote_volume: Decimal,

    // 当前时间
    pub ts: i64,

}

impl CoinThumb {
    pub fn new(symbol: String, open: Decimal, high: Decimal, low: Decimal, close: Decimal, chg: Decimal, change: Decimal, volume: Decimal, quote_volume: Decimal) -> Self {
        Self {
            symbol,
            icon: "".to_string(),
            open,
            high,
            low,
            close,
            chg,
            change,
            volume,
            quote_volume,
            ts: 0,
        }
    }
}