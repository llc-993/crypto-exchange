use common::models::{Kline, MarketType};
use common::enums::KlineInterval;
use common::pulsar::{PulsarClient, topics};
use common::cache;
use std::time::{SystemTime, UNIX_EPOCH};
use common::models::coin_thumb_price::CoinThumbPrice;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Redis Hash 字段数量缓存
/// Key: redis_key, Value: 字段数量
static HASH_COUNT_CACHE: Lazy<Mutex<HashMap<String, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 为所有时间间隔生成K线
///
/// 这是主入口函数，会遍历所有K线间隔并生成对应的K线数据
pub async fn generate_all_klines(thumb_price: &CoinThumbPrice) {
    // 为所有时间间隔生成K线
    for interval in KlineInterval::all() {
        generate_kline(thumb_price, interval).await;
    }
}

/// 生成K线数据
///
/// 逻辑流程：
/// 1. 根据时间间隔计算当前K线的开始时间戳（确保正确对齐）
/// 2. 从Redis获取现有K线数据，如果不存在则创建新的
/// 3. 更新K线的OHLC数据
/// 4. 检查K线是否已完成（时间窗口切换）
/// 5. 存储更新后的K线到Redis
/// 6. 每次更新后都实时推送到Pulsar（无论是否完成）
///
/// 存储kline的逻辑流程
///
/// hset 存储到 Redis
///     ↓
/// 检查 map 中是否有该 redis_key
///     ↓
/// 没有？ → 是 → 调用 hgetall 初始化 map
///     ↓              ↓
///    否             更新 map = all_data.len()
///     ↓              ↓
/// 检查 count > 100? → 是 → 处理批量存储
///     ↓              ↓
///    否              更新 map（-50）
///     ↓
/// 更新 map（+1）
///
pub async fn generate_kline(thumb_price: &CoinThumbPrice, interval: KlineInterval) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 计算当前K线的开始和结束时间戳
    let (open_time, close_time) = calculate_kline_timestamp(now, interval);

    // 构建Redis key: kline:{market_type}:{symbol}:{interval}:{open_time}
    let redis_key = format!(
        "kline:{:?}:{}:{}",
        thumb_price.market_type,
        thumb_price.symbol,
        interval.interval()
    );

    // 从Redis Hash获取现有K线（使用open_time作为field）
    let mut kline = match cache.hget(&redis_key, &open_time.to_string()).await {
        Ok(Some(json_str)) => {
            // 反序列化现有K线
            match serde_json::from_str::<Kline>(&json_str) {
                Ok(mut k) => {
                    // 检查是否时间窗口已切换
                    if k.close_time < now {
                        // 时间窗口已切换，创建新K线
                        create_new_kline(thumb_price, interval, open_time, close_time).await
                    } else {
                        // 更新现有K线
                        k.update_price(thumb_price.price);
                        k
                    }
                }
                Err(e) => {
                    log::warn!("[Kline] 反序列化失败，创建新K线: {}", e);
                    create_new_kline(thumb_price, interval, open_time, close_time).await
                }
            }
        }
        Ok(None) => {
            // Redis Hash中没有该open_time的K线，创建新K线
            create_new_kline(thumb_price, interval, open_time, close_time).await
        }
        Err(e) => {
            log::error!("[Kline] Redis获取失败: key={}, error={}", redis_key, e);
            // Redis错误时也创建新K线，避免阻塞
            create_new_kline(thumb_price, interval, open_time, close_time).await
        }
    };

    // 检查K线是否完成（当前时间已超过close_time）
    let is_closed = now >= close_time;
    if is_closed && !kline.is_closed {
        kline.mark_closed();
    }

    // 每次更新后都实时推送到Pulsar（无论K线是否完成）
  /*  let topic = match thumb_price.market_type {
        MarketType::Spot => topics::kline::SPOT_KLINE,
        MarketType::Futures => topics::kline::FUTURES_KLINE,
    };*/

   // PulsarClient::publish_async(topic, kline.clone());
    log::debug!(
        "[Kline] 实时推送K线: {} {:?} {} 价格: {} 状态: {}",
        kline.symbol,
        kline.market_type,
        kline.interval,
        kline.close,
        if kline.is_closed { "已完成" } else { "进行中" }
    );

    // 存储更新后的K线到Redis
    match serde_json::to_string(&kline) {
        Ok(json_str) => {
            if let Err(e) = cache.hset(&redis_key, &kline.open_time.to_string(), &json_str).await {
                log::error!("[Kline] Redis存储失败: {}", e);
                return;
            }

            // 检查map中是否有该key，以及是否需要检查hash字段数量
            let (need_init, need_check) = {
                let cache_map = HASH_COUNT_CACHE.lock().unwrap();
                let count = cache_map.get(&redis_key).copied();
                // 如果map中没有该key，需要初始化
                let init = count.is_none();
                // 如果count > 100，需要检查并处理
                let check = count.unwrap_or(0) > 100;
                (init, check)
            };

            if need_init || need_check {
                // 需要初始化或检查，调用hgetall
                // 获取所有数据并初始化/更新缓存
                if let Ok(all_data) = cache.hgetall(&redis_key).await {
                    let count = all_data.len();
                    
                    // 更新缓存（初始化或更新）
                    {
                        let mut cache_map = HASH_COUNT_CACHE.lock().unwrap();
                        cache_map.insert(redis_key.clone(), count);
                    }

                    if count > 2 {
                        // 解析所有Kline数据并按时间排序
                        let mut klines: Vec<Kline> = all_data
                            .into_iter()
                            .filter_map(|(open_time_str, json_str)| {
                                open_time_str.parse::<i64>()
                                    .ok()
                                    .and_then(|_| serde_json::from_str::<Kline>(&json_str).ok())
                            })
                            .collect();
                        
                        // 按open_time排序，取最老的50条
                        klines.sort_by_key(|k| k.open_time);
                        let oldest_50: Vec<Kline> = klines.into_iter().take(50).collect();
                        
                        // 存入MongoDB
                        if let Err(e) = Kline::insert_many(oldest_50.clone()).await {
                            log::error!("[Kline] MongoDB批量存储失败: {}", e);
                        } else {
                            // 从Redis删除已存入MongoDB的数据
                            let fields_to_delete: Vec<String> = oldest_50.iter()
                                .map(|k| k.open_time.to_string())
                                .collect();
                            let fields_refs: Vec<&str> = fields_to_delete.iter().map(|s| s.as_str()).collect();
                            if let Err(e) = cache.hdel(&redis_key, &fields_refs).await {
                                log::warn!("[Kline] 删除Redis数据失败: {}", e);
                            } else {
                                // 更新缓存计数（-50）
                                let mut cache_map = HASH_COUNT_CACHE.lock().unwrap();
                                if let Some(c) = cache_map.get_mut(&redis_key) {
                                    *c = c.saturating_sub(50);
                                }
                            }
                        }
                    }
                }
            } else {
                // map中有该key且 <= 100，只更新计数（+1）
                let mut cache_map = HASH_COUNT_CACHE.lock().unwrap();
                if let Some(count) = cache_map.get_mut(&redis_key) {
                    *count += 1;
                }
            }
        }
        Err(e) => {
            log::error!("[Kline] 序列化失败: {}", e);
        }
    }
}

/// 创建新K线
async fn create_new_kline(
    thumb_price: &CoinThumbPrice,
    interval: KlineInterval,
    open_time: i64,
    close_time: i64,
) -> Kline {
    Kline::new(
        thumb_price.symbol.clone(),
        thumb_price.market_type.clone(),
        interval,
        open_time,
        close_time,
        thumb_price.price,
    )
}

/// 计算K线的时间戳
///
/// 确保K线时间正确对齐到间隔边界
///
/// 示例（5分钟K线）：
/// - 18分钟的数据 → open_time = 15分钟（15-19分钟）
/// - 23分钟的数据 → open_time = 20分钟（20-24分钟）
/// - 24分钟的数据 → open_time = 20分钟（20-24分钟）
///
/// 返回 (open_time, close_time)
fn calculate_kline_timestamp(timestamp: i64, interval: KlineInterval) -> (i64, i64) {
    let seconds = get_interval_seconds(interval);

    // 向下取整到间隔边界
    // 例如：5分钟K线，18分钟 = 1080秒 → (1080/300)*300 = 900秒 = 15分钟
    let open_time = (timestamp / seconds) * seconds;

    // 结束时间 = 开始时间 + 间隔 - 1秒
    // 例如：15分钟开始，5分钟间隔 → 15:00 - 15:04:59
    let close_time = open_time + seconds - 1;

    (open_time, close_time)
}

/// 获取时间间隔对应的秒数
fn get_interval_seconds(interval: KlineInterval) -> i64 {
    match interval {
        KlineInterval::Min1 => 60,
        KlineInterval::Min5 => 300,
        KlineInterval::Min15 => 900,
        KlineInterval::Min30 => 1800,
        KlineInterval::Hour1 => 3600,
        KlineInterval::Day1 => 86400,
        KlineInterval::Week1 => 604800,
        KlineInterval::Month1 => 2592000,  // 30天
    }
}