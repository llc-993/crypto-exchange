use serde_json::Value;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::models::{UnifiedTicker, MarketType};

use std::time::{SystemTime, UNIX_EPOCH};
/// Ticker 数据转换器
pub struct TickerConverter;

impl TickerConverter {
    /// 从 OKX Futures Ticker 数据转换
    pub fn from_okx_futures(data: &Value, symbol: &str) -> Result<UnifiedTicker, String> {
        let mut ticker = UnifiedTicker::new(
            "okx".to_string(),
            symbol.to_string(),
            MarketType::Futures,
        );

        // 解析价格数据
        ticker.close = Self::parse_decimal(data, "last")?;
        ticker.open = Self::parse_decimal(data, "open24h")?;
        ticker.high = Self::parse_decimal(data, "high24h")?;
        ticker.low = Self::parse_decimal(data, "low24h")?;
        
        // 解析成交量
        ticker.volume_24h = Self::parse_decimal(data, "vol24h")?;
        ticker.quote_volume_24h = Self::parse_decimal(data, "volCcy24h")?;
        
        // 解析买卖价
        ticker.bid_price = Self::parse_optional_decimal(data, "bidPx");
        ticker.ask_price = Self::parse_optional_decimal(data, "askPx");
        
        // 解析时间戳
        ticker.timestamp = Self::parse_timestamp(data, "ts")?;
        
        // 自动计算涨跌幅
        ticker.calculate_change_percent();
        
        Ok(ticker)
    }

    /// 从 OKX Spot Ticker 数据转换
    pub fn from_okx_spot(data: &Value, symbol: &str) -> Result<UnifiedTicker, String> {
        let mut ticker = UnifiedTicker::new(
            "okx".to_string(),
            symbol.to_string(),
            MarketType::Spot,
        );

        ticker.close = Self::parse_decimal(data, "last")?;
        ticker.open = Self::parse_decimal(data, "open24h")?;
        ticker.high = Self::parse_decimal(data, "high24h")?;
        ticker.low = Self::parse_decimal(data, "low24h")?;
        ticker.volume_24h = Self::parse_decimal(data, "vol24h")?;
        ticker.quote_volume_24h = Self::parse_decimal(data, "volCcy24h")?;
        ticker.bid_price = Self::parse_optional_decimal(data, "bidPx");
        ticker.ask_price = Self::parse_optional_decimal(data, "askPx");
        ticker.timestamp = Self::parse_timestamp(data, "ts")?;
        ticker.calculate_change_percent();
        
        Ok(ticker)
    }

    /// 从 Binance Futures Ticker 数据转换
    pub fn from_binance_futures(data: &Value, symbol: &str) -> Result<UnifiedTicker, String> {
        let mut ticker = UnifiedTicker::new(
            "binance".to_string(),
            symbol.to_string(),
            MarketType::Futures,
        );

        ticker.close = Self::parse_decimal(data, "c")?;
        ticker.open = Self::parse_decimal(data, "o")?;
        ticker.high = Self::parse_decimal(data, "h")?;
        ticker.low = Self::parse_decimal(data, "l")?;
        ticker.volume_24h = Self::parse_decimal(data, "v")?;
        ticker.quote_volume_24h = Self::parse_decimal(data, "q")?;
        ticker.timestamp = Self::parse_timestamp(data, "E")?;
        ticker.calculate_change_percent();
        
        Ok(ticker)
    }

    /// 从 Binance Spot Ticker 数据转换
    pub fn from_binance_spot(data: &Value, symbol: &str) -> Result<UnifiedTicker, String> {
        let mut ticker = UnifiedTicker::new(
            "binance".to_string(),
            symbol.to_string(),
            MarketType::Spot,
        );

        ticker.close = Self::parse_decimal(data, "c")?;
        ticker.open = Self::parse_decimal(data, "o")?;
        ticker.high = Self::parse_decimal(data, "h")?;
        ticker.low = Self::parse_decimal(data, "l")?;
        ticker.volume_24h = Self::parse_decimal(data, "v")?;
        ticker.quote_volume_24h = Self::parse_decimal(data, "q")?;
        ticker.timestamp = Self::parse_timestamp(data, "E")?;
        ticker.calculate_change_percent();
        
        Ok(ticker)
    }

    /// 从 Bitget Futures Ticker 数据转换
    pub fn from_bitget_futures(data: &Value, symbol: &str) -> Result<UnifiedTicker, String> {
        let mut ticker = UnifiedTicker::new(
            "bitget".to_string(),
            symbol.to_string(),
            MarketType::Futures,
        );

        ticker.close = Self::parse_decimal(data, "lastPr")?;
        ticker.open = Self::parse_decimal(data, "open24h")?;
        ticker.high = Self::parse_decimal(data, "high24h")?;
        ticker.low = Self::parse_decimal(data, "low24h")?;
        ticker.volume_24h = Self::parse_decimal(data, "baseVolume")?;
        ticker.quote_volume_24h = Self::parse_decimal(data, "quoteVolume")?;
        ticker.bid_price = Self::parse_optional_decimal(data, "bidPr");
        ticker.ask_price = Self::parse_optional_decimal(data, "askPr");
        ticker.timestamp = Self::parse_timestamp(data, "ts")?;
        ticker.calculate_change_percent();
        
        Ok(ticker)
    }

    /// 从 Bitget Spot Ticker 数据转换
    pub fn from_bitget_spot(data: &Value, symbol: &str) -> Result<UnifiedTicker, String> {
        let mut ticker = UnifiedTicker::new(
            "bitget".to_string(),
            symbol.to_string(),
            MarketType::Spot,
        );

        ticker.close = Self::parse_decimal(data, "lastPr")?;
        ticker.open = Self::parse_decimal(data, "open")?;
        ticker.high = Self::parse_decimal(data, "high")?;
        ticker.low = Self::parse_decimal(data, "low")?;
        ticker.volume_24h = Self::parse_decimal(data, "baseVolume")?;
        ticker.quote_volume_24h = Self::parse_decimal(data, "quoteVolume")?;
        ticker.bid_price = Self::parse_optional_decimal(data, "bestBid");
        ticker.ask_price = Self::parse_optional_decimal(data, "bestAsk");
        ticker.timestamp = Self::parse_timestamp(data, "ts")?;
        ticker.calculate_change_percent();
        
        Ok(ticker)
    }

    // ========== 辅助方法 ==========

    /// 解析必需的 Decimal 字段
    fn parse_decimal(data: &Value, field: &str) -> Result<Decimal, String> {
        data.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("缺少字段: {}", field))
            .and_then(|s| {
                Decimal::from_str(s)
                    .map_err(|e| format!("解析 {} 失败: {}", field, e))
            })
    }

    /// 解析可选的 Decimal 字段
    fn parse_optional_decimal(data: &Value, field: &str) -> Option<Decimal> {
        data.get(field)
            .and_then(|v| v.as_str())
            .and_then(|s| Decimal::from_str(s).ok())
    }

    /// 解析时间戳（毫秒）
    fn parse_timestamp(data: &Value, field: &str) -> Result<i64, String> {
        data.get(field)
            .and_then(|v| {
                // 尝试作为字符串解析
                if let Some(s) = v.as_str() {
                    s.parse::<i64>().ok()
                } else {
                    // 尝试作为数字解析
                    v.as_i64()
                }
            })
            .ok_or_else(|| format!("缺少或无效的时间戳字段: {}", field))
    }
}

/// Mark Price 数据转换器
pub struct MarkPriceConverter;

impl MarkPriceConverter {
    /// 从 Binance Futures Mark Price 数据转换
    pub fn from_binance_futures(data: &Value, symbol: &str) -> Result<crate::models::UnifiedMarkPrice, String> {
        let mark_price = Self::parse_decimal(data, "p")?;
        let index_price = Self::parse_decimal(data, "i")?;
        let funding_rate = Self::parse_optional_decimal(data, "r");
        let next_funding_time = Self::parse_i64(data, "T").ok();
        
        Ok(crate::models::UnifiedMarkPrice {
            exchange: "binance".to_string(),
            symbol: symbol.to_string(),
            mark_price,
            index_price,
            funding_rate: funding_rate, // funding_rate is already Option<Decimal>
            funding_time: None,
            next_funding_time,
            timestamp: Self::current_timestamp_millis(),
        })
    }

    /// 从 OKX Futures Mark Price 数据转换
    pub fn from_okx_futures(data: &Value, symbol: &str) -> Result<crate::models::UnifiedMarkPrice, String> {
        let mark_price = Self::parse_decimal(data, "markPx")?;
        let index_price = Self::parse_decimal(data, "idxPx")?;
        
        Ok(crate::models::UnifiedMarkPrice {
            exchange: "okx".to_string(),
            symbol: symbol.to_string(),
            mark_price,
            index_price,
            funding_rate: None,
            funding_time: None,
            next_funding_time: None,
            timestamp: Self::current_timestamp_millis(),
        })
    }

    /// 从 Bitget Futures Mark Price 数据转换
    /// Bitget ticker 包含: markPrice, indexPrice, fundingRate
    pub fn from_bitget_futures(data: &Value, symbol: &str) -> Result<crate::models::UnifiedMarkPrice, String> {
        let mark_price = Self::parse_decimal(data, "markPrice")?;
        let index_price = Self::parse_decimal(data, "indexPrice")?;
        let funding_rate = Self::parse_optional_decimal(data, "fundingRate");
        let next_funding_time = Self::parse_i64(data, "nextFundingTime")?;
        
        Ok(crate::models::UnifiedMarkPrice {
            exchange: "bitget".to_string(),
            symbol: symbol.to_string(),
            mark_price,
            index_price,
            funding_rate,
            funding_time: None,
            next_funding_time: Some(next_funding_time),
            timestamp: Self::current_timestamp_millis(),
        })
    }

    fn parse_decimal(data: &Value, field: &str) -> Result<Decimal, String> {
        data.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing field: {}", field))
            .and_then(|s| Decimal::from_str(s)
                .map_err(|e| format!("Failed to parse {}: {}", field, e)))
    }

    fn parse_optional_decimal(data: &Value, field: &str) -> Option<Decimal> {
        data.get(field)
            .and_then(|v| v.as_str())
            .and_then(|s| Decimal::from_str(s).ok())
    }

    /// 从 OKX Futures Funding Rate 数据转换为 UnifiedMarkPrice
    /// 用于处理 funding-rate 频道的数据
    pub fn from_okx_funding_rate(data: &Value, symbol: &str) -> Result<crate::models::UnifiedMarkPrice, String> {
        let funding_rate = Self::parse_decimal(data, "fundingRate")?;
        let funding_time = Self::parse_i64(data, "fundingTime")?;
        let next_funding_time = Self::parse_i64(data, "nextFundingTime")?;
        
        Ok(crate::models::UnifiedMarkPrice {
            exchange: "okx".to_string(),
            symbol: symbol.to_string(),
            mark_price: Decimal::ZERO,
            index_price: Decimal::ZERO,
            funding_rate: Some(funding_rate),
            funding_time: Some(funding_time),
            next_funding_time: Some(next_funding_time),
            timestamp: Self::current_timestamp_millis(),
        })
    }

    fn parse_i64(data: &Value, field: &str) -> Result<i64, String> {
        data.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing field: {}", field))
            .and_then(|s| s.parse::<i64>()
                .map_err(|e| format!("Failed to parse {}: {}", field, e)))
    }

    fn current_timestamp_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

