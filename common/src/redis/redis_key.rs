//! Redis Key 常量定义
//!
//! 统一管理所有 Redis key，便于维护和查找

/// Ticker 相关 Key
pub mod ticker {
    /// 现货 Ticker 缓存 (Hash)
    /// Field: symbol, Value: JSON(UnifiedTicker)
    pub const SPOT_TICKER: &str = "ticker:spot";

    /// 永续合约 Ticker 缓存 (Hash)
    /// Field: exchange:symbol, Value: JSON(UnifiedTicker)
    pub const FUTURES_TICKER: &str = "ticker:futures";
}

/// MarkPrice 相关 Key
pub mod mark_price {
    /// 标记价格缓存 (Hash)
    /// Field: exchange:symbol, Value: JSON(UnifiedMarkPrice)
    pub const MARK_PRICE: &str = "mark_price:raw";

    /// 聚合后的标记价格缓存 (Hash)
    /// Field: symbol, Value: JSON(UnifiedMarkPrice)
    pub const AGGREGATED_MARK_PRICE: &str = "mark_price:aggregated";
}

pub mod kline {
    /// k线
    pub const KLINE_KEY: &str = "kline:cache";
}

/// MongoDB Collection 相关 Key
pub mod mongodb {
    /// MongoDB Collection 存在性缓存 (Hash)
    /// Field: collection_name, Value: "1" (表示存在)
    pub const COLLECTION_EXISTS: &str = "mongodb:collections";
}
