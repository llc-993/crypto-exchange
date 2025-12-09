use disruptor::Sequence;
use rust_decimal::Decimal;
use common::models::{coin_thumb_price, UnifiedTicker};
use crate::common::{price_cache, ticker_cache};
use crate::common::ticker_cache::calculate_weighted_futures_price;
use common::models::MarketType;
use common::pulsar::topics;
use common::PulsarClient;

/// Ticker 事件数据结构
/// 在 Disruptor RingBuffer 中流转
#[derive(Debug, Clone, Default)]
pub struct UnifiedTickerEvent {
    pub ticker: UnifiedTicker,
}

/// Disruptor 消费者处理函数
/// 此函数在后台线程中被调用，处理从 ticker_consumer 推送过来的 Ticker 数据
pub fn handle_ticker_event(event: &UnifiedTickerEvent, _sequence: Sequence, _end_of_batch: bool) {
    // 处理接收到的 Ticker 数据
    let mut ticker = event.ticker.clone();

    // 统一处理 symbol，去掉 "-" 和 "/"
    ticker.symbol = ticker.symbol.replace("-", "").replace("/", "");

    // 存储 Ticker 到缓存
    ticker_cache::set_ticker(ticker.clone());

    // 1. 聚合计算（如计算加权平均价格、成交量统计等）
    // 计算最终价格：Futures 使用加权价格，Spot 使用当前价格
    let price = match ticker.market_type {
        MarketType::Futures => {
            calculate_weighted_futures_price(&ticker.symbol)
                .map(|(weighted_price, exchange_count)| {
                    #[cfg(debug_assertions)]
                    {
                        log::debug!("[Futures加权价格] {} 加权平均价格: {} (来自 {} 个交易所)", 
                            ticker.symbol, weighted_price, exchange_count);
                    }
                    weighted_price
                })
                .unwrap_or_else(|| {
                    log::debug!("[Futures加权价格] {} 无法计算加权价格，使用当前价格: {}",
                        ticker.symbol, ticker.close);
                    ticker.close
                })
        }
        MarketType::Spot => ticker.close,
    };

    // 2. 价格调整（可选：添加点差、手续费等）
    let adjusted_price = apply_price_adjustment(&ticker.symbol, price, &ticker.market_type);

    // 3. 存储最终价格并检测变化
    price_cache::set_final_price(&ticker.symbol, adjusted_price, &ticker.market_type);


    // 异步处理订单撮合
    let symbol = ticker.symbol.clone();
    let market_type = ticker.market_type.clone();

    matching_order(&symbol, adjusted_price, &market_type);


    handle_price_change(&symbol, adjusted_price, market_type);

    // 处理 ticker 数据的核心逻辑
    // 例如：更新内存中的价格缓存、触发价格变化通知等
    // 注意：这里应该避免阻塞操作，保持高性能
}

/// 价格调整（点差、手续费等）
fn apply_price_adjustment(_symbol: &str, price: Decimal, _market_type: &MarketType) -> Decimal {
    // TODO: 从配置读取调整参数
    // 示例：可以根据 symbol 或 market_type 添加不同的调整逻辑
    price
}

/// 处理订单撮合
fn matching_order(symbol: &str, price: Decimal, market_type: &MarketType) {
    // TODO: 从数据库/缓存获取待撮合订单
    // 1. 止损单检查
    // 2. 止盈单检查  
    // 3. 限价单撮合
    #[cfg(debug_assertions)]
    log::debug!("[撮合] {} {:?} 价格: {}", symbol, market_type, price);
}

/// 将数据推送到 market 服务
fn handle_price_change(symbol: &str, price: Decimal, market_type: MarketType) {
    // 推送到 market 服务生成 K 线、行情
    let thumb_price = coin_thumb_price::CoinThumbPrice::new(symbol.to_string(), price, market_type);

    PulsarClient::publish_async(topics::ticker::EX_THUMB_PRICE, thumb_price);
}

