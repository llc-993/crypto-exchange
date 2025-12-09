/// Pulsar Topic 常量定义
/// 
/// 集中管理所有 Pulsar Topic 名称，方便维护和修改

/// Ticker 相关 Topics
pub mod ticker {
    /// 现货 Ticker 数据
    pub const SPOT_TICKER: &str = "spot-ticker";
    
    /// 合约 Ticker 数据
    pub const FUTURES_TICKER: &str = "futures-ticker";

    // 在撮合推出来的交易数据
    pub const EX_THUMB_PRICE: &str = "ex-thumb-price";
}

/// Mark Price 相关 Topics
pub mod mark_price {
    /// 合约标记价格数据
    pub const FUTURES_MARK_PRICE: &str = "futures-mark-price";
}

/// Funding Rate Topics
pub mod funding_rate {
    /// Futures Funding Rate Topic
    pub const FUTURES_FUNDING_RATE: &str = "futures-funding-rate";
}

/// Kline 相关 Topics
pub mod kline {
    /// 现货 K线数据
    pub const SPOT_KLINE: &str = "spot-kline";
    
    /// 合约 K线数据
    pub const FUTURES_KLINE: &str = "futures-kline";
}

/// Depth 相关 Topics
pub mod depth {
    /// 现货深度数据
    pub const SPOT_DEPTH: &str = "spot-depth";
    
    /// 合约深度数据
    pub const FUTURES_DEPTH: &str = "futures-depth";
}

/// 所有 Topics 列表（用于批量操作）
pub const ALL_TOPICS: &[&str] = &[
    ticker::SPOT_TICKER,
    ticker::FUTURES_TICKER,
    mark_price::FUTURES_MARK_PRICE,
    funding_rate::FUTURES_FUNDING_RATE,
    kline::SPOT_KLINE,
    kline::FUTURES_KLINE,
    depth::SPOT_DEPTH,
    depth::FUTURES_DEPTH,
];
