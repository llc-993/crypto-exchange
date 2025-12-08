use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 统一的标记价格数据模型
/// 包含标记价格、指数价格、资金费率等信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMarkPrice {
    /// 交易所名称
    pub exchange: String,
    /// 交易对符号
    pub symbol: String,
    /// 标记价格
    pub mark_price: Decimal,
    /// 指数价格
    pub index_price: Decimal,
    /// 资金费率（可选，仅永续合约有）
    pub funding_rate: Option<Decimal>,
    /// 资金费率结算时间（毫秒时间戳，可选）
    pub funding_time: Option<i64>,
    /// 下次资金费率结算时间（毫秒时间戳，可选）
    pub next_funding_time: Option<i64>,
    /// 数据时间戳（毫秒）
    pub timestamp: i64,
}

impl UnifiedMarkPrice {
    /// 创建新的 UnifiedMarkPrice
    pub fn new(exchange: String, symbol: String) -> Self {
        Self {
            exchange,
            symbol,
            mark_price: Decimal::ZERO,
            index_price: Decimal::ZERO,
            funding_rate: None,
            funding_time: None,
            next_funding_time: None,
            timestamp: 0,
        }
    }
}

/// 实现 Pulsar 消息反序列化
impl pulsar::DeserializeMessage for UnifiedMarkPrice {
    type Output = Result<UnifiedMarkPrice, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
