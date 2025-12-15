use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::MarketType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoinThumbPrice {
    // 交易对
    pub symbol: String,

    // 现价
    pub price: Decimal,

    // 成交量（可选，兼容旧数据）
    #[serde(default)]
    pub volume: Decimal,

    // 市场类型
    pub market_type: MarketType,
}

impl CoinThumbPrice {
    pub fn new(symbol: String, price: Decimal, volume: Decimal, market_type: MarketType) -> Self {
        Self {
            symbol,
            price,
            volume,
            market_type,
        }
    }
}

impl pulsar::DeserializeMessage for CoinThumbPrice {
    type Output = Result<CoinThumbPrice, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}