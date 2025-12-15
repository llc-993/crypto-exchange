use common::models::{order_book_depth::{OrderBookDepth, Level}, coin_thumb_price::CoinThumbPrice, MarketType};
use rust_decimal::Decimal;
use std::str::FromStr;
use rand::Rng;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// 生成盘口深度数据
///
/// 根据当前价格生成20档买卖盘口数据
pub async fn generate_orderbook_depth(thumb_price: &CoinThumbPrice) {
    // 使用当前价格作为中间价
    let mid_price = thumb_price.price;

    // 随机生成价格精度（tick_size），范围在 0.0001 到 1.0 之间
    let tick_size = generate_random_tick_size();

    // 随机生成衰减因子，范围在 0.01 到 1.0 之间
    let factor = generate_random_factor();

    // 生成20档深度
    let order_book = generate_depth(mid_price, tick_size, factor);

    // TODO: 推送盘口数据到Pulsar
    // let topic = match thumb_price.market_type {
    //     MarketType::Spot => common::pulsar::topics::depth::SPOT_DEPTH,
    //     MarketType::Futures => common::pulsar::topics::depth::FUTURES_DEPTH,
    // };
    // common::PulsarClient::publish_async(topic, order_book);

    log::debug!(
        "[Depth] {} {:?} 生成盘口 - 中间价: {} 买盘档数: {} 卖盘档数: {}",
        thumb_price.symbol,
        thumb_price.market_type,
        mid_price,
        order_book.bids.len(),
        order_book.asks.len()
    );
}

/// 随机生成价格精度（tick_size）
/// 范围：0.0001 到 1.0
fn generate_random_tick_size() -> Decimal {
    let mut rng = rand::rng();
    let min = Decimal::from_str("0.0001").unwrap();
    let max = Decimal::from_str("1.0").unwrap();

    let r: f64 = rng.random();
    let tick_size = min + (max - min) * Decimal::from_f64(r).unwrap();

    tick_size.round_dp(6)
}

/// 随机生成衰减因子（factor）
/// 范围：0.01 到 1.0
fn generate_random_factor() -> Decimal {
    let mut rng = rand::rng();
    let min = Decimal::from_str("0.01").unwrap();
    let max = Decimal::from_str("1.0").unwrap();

    let r: f64 = rng.random();
    let factor = min + (max - min) * Decimal::from_f64(r).unwrap();

    factor.round_dp(6)
}

fn generate_random_offset() -> Decimal {
    let mut rng = rand::rng();
    let min = Decimal::from_str("0.0001").unwrap();
    let max = Decimal::from_str("0.002").unwrap();

    let r: f64 = rng.random();
    let factor = min + (max - min) * Decimal::from_f64(r).unwrap();

    factor.round_dp(6)
}

/// 指数衰减的随机数生成
pub fn exponential_random_decimal(factor: Decimal) -> Decimal {
    let mut rng = rand::rng();

    // max 范围
    let max_min = Decimal::from_str("0.0001").unwrap();
    let max_max = Decimal::from_str("10000").unwrap();

    // 1. max ∈ [max_min , max_max]
    let r1: f64 = rng.random();
    let max = max_min + (max_max - max_min) * Decimal::from_f64(r1).unwrap();

    // 2. r ∈ [0,1]
    let r2: f64 = rng.random();
    let r_dec = Decimal::from_f64(r2).unwrap();

    // 3. decay = exp(-factor)
    let decay = (-factor).to_f64().unwrap().exp(); // 用 f64 exp
    let decay_dec = Decimal::from_f64(decay).unwrap();

    // 4. 结果
    let v = max * r_dec * decay_dec;

    // 5. 保留 6 位小数
    v.round_dp(6)
}

/// 生成 20 档深度
pub fn generate_depth(
    mid_price: Decimal,
    tick_size: Decimal,
    factor: Decimal,
) -> OrderBookDepth {
    let mut bids = Vec::with_capacity(20);
    let mut asks = Vec::with_capacity(20);

    for i in 1..=20 {
        // 价格偏移
        let offset = generate_random_offset() + (tick_size * Decimal::from_i32(i as i32).unwrap());

        // Bid：逐渐降低
        let bid_price = mid_price - offset;
        let bid_amount = exponential_random_decimal(factor);

        let offset1 = generate_random_offset() + (tick_size * Decimal::from_i32(i as i32).unwrap());
        // Ask：逐渐升高
        let ask_price = mid_price + offset1;
        let ask_amount = exponential_random_decimal(factor);

        bids.push(Level {
            price: bid_price.round_dp(6),
            amount: bid_amount,
        });

        asks.push(Level {
            price: ask_price.round_dp(6),
            amount: ask_amount,
        });
    }

    OrderBookDepth { bids, asks }
}
