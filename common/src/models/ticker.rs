use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// 市场类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarketType {
    /// 现货
    #[default]
    Spot,
    /// 永续合约
    Futures,
}

/// 统一的 Ticker 数据结构
/// 适用于所有交易所的现货和合约市场
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UnifiedTicker {
    /// 交易所名称 (binance, okx, bitget, bybit等)
    pub exchange: String,
    
    /// 交易对 (BTC-USDT, BTCUSDT等)
    pub symbol: String,
    
    /// 市场类型 (spot/futures)
    pub market_type: MarketType,
    
    /// 最新成交价
    pub close: Decimal,
    
    /// 24小时开盘价
    pub open: Decimal,
    
    /// 24小时最高价
    pub high: Decimal,
    
    /// 24小时最低价
    pub low: Decimal,
    
    /// 24小时成交量 (基础货币)
    pub volume_24h: Decimal,
    
    /// 24小时成交额 (计价货币)
    pub quote_volume_24h: Decimal,
    
    /// 24小时价格变化量
    pub change_24h: Option<Decimal>,
    
    /// 24小时涨跌幅 (百分比)
    pub change_percent_24h: Option<Decimal>,
    
    /// 买一价
    pub bid_price: Option<Decimal>,
    
    /// 卖一价
    pub ask_price: Option<Decimal>,
    
    /// 时间戳 (毫秒)
    pub timestamp: i64,
}

impl UnifiedTicker {
    /// 创建新的 Ticker
    pub fn new(exchange: String, symbol: String, market_type: MarketType) -> Self {
        Self {
            exchange,
            symbol,
            market_type,
            close: Decimal::ZERO,
            open: Decimal::ZERO,
            high: Decimal::ZERO,
            low: Decimal::ZERO,
            volume_24h: Decimal::ZERO,
            quote_volume_24h: Decimal::ZERO,
            change_24h: None,
            change_percent_24h: None,
            bid_price: None,
            ask_price: None,
            timestamp: 0,
        }
    }
    
    /// 计算涨跌幅
    pub fn calculate_change_percent(&mut self) {
        if self.open > Decimal::ZERO {
            let change = self.close - self.open;
            self.change_24h = Some(change);
            self.change_percent_24h = Some((change / self.open) * Decimal::from(100));
        }
    }
}

impl pulsar::DeserializeMessage for UnifiedTicker {
    type Output = Result<UnifiedTicker, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
