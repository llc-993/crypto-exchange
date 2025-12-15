use common::models::{Kline, coin_thumb_price::CoinThumbPrice, coin_thumb::CoinThumb};
use common::enums::KlineInterval;
use common::cache;
use rust_decimal::Decimal;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// 行情统计数据缓存
/// Key: {symbol}:{market_type}, Value: (last_query_time, CoinThumb)
static THUMB_STATS_CACHE: Lazy<Mutex<HashMap<String, (i64, CoinThumb)>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 生成行情统计数据
/// 
/// 查询24小时内的K线数据（从Redis和MongoDB聚合），计算统计数据
/// 使用缓存优化，避免频繁查询
pub async fn generate_thumb_stats(thumb_price: &CoinThumbPrice) {
    // 获取当前时间戳（秒）
    let current_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 获取当前分钟的开始时间戳（将秒数部分清零）
    // 例如：01:34:45 -> 01:34:00
    let now = current_secs - (current_secs % 60);
    
    // 24小时前的时间戳
    let start_time = now - 86400; // 24小时 = 86400秒
    
    // 构建缓存key
    let cache_key = format!("{}:{:?}", thumb_price.symbol, thumb_price.market_type);
    
    // 检查缓存，如果最近查询过（比如1分钟内），使用缓存但更新当前价格
    let should_query = {
        let stats_cache = THUMB_STATS_CACHE.lock().unwrap();
        if let Some((last_time, _)) = stats_cache.get(&cache_key) {
            // 如果上次查询时间在1分钟内，不重新查询，但需要更新价格
            let time_diff = now - last_time;
            if time_diff < 60 {
                false  // 使用缓存，但需要更新价格
            } else {
                true   // 超过1分钟，重新查询
            }
        } else {
            true  // 缓存不存在，需要查询
        }
    };
    
    if !should_query {
        // 使用缓存数据，但更新当前价格相关的字段（close、chg、change）
        let mut stats_cache = THUMB_STATS_CACHE.lock().unwrap();
        if let Some((_, mut coin_thumb)) = stats_cache.remove(&cache_key) {
            // 更新收盘价
            coin_thumb.close = thumb_price.price;
            
            // 重新计算价格变化量（chg）
            coin_thumb.chg = thumb_price.price - coin_thumb.open;
            
            // 重新计算涨跌幅（百分比，change）
            coin_thumb.change = if coin_thumb.open > Decimal::ZERO {
                (coin_thumb.chg / coin_thumb.open) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };
            
            // 更新时间戳
            coin_thumb.ts = now;
            
            // 如果当前价格超过最高价或低于最低价，更新
            if thumb_price.price > coin_thumb.high {
                coin_thumb.high = thumb_price.price;
            }
            if thumb_price.price < coin_thumb.low {
                coin_thumb.low = thumb_price.price;
            }
            
            // 累加当前交易的成交量和成交额
            coin_thumb.volume += thumb_price.volume;
            coin_thumb.quote_volume += thumb_price.price * thumb_price.volume;
            
            // 更新缓存
            stats_cache.insert(cache_key.clone(), (now, coin_thumb.clone()));

            // 使用缓存，直接返回
            return;
        }
        // 如果缓存不存在，继续执行查询逻辑
    }
    
    // 如果需要查询（缓存不存在或已过期）
    // 查询24小时内的K线数据
    match query_24h_klines(thumb_price, start_time, now).await {
        Ok(klines) => {
            // 计算统计数据
            let coin_thumb = calculate_thumb_stats(&klines, thumb_price.price, &thumb_price.symbol, now);
            
            // 记录日志
            log::debug!(
                "[ThumbStats] {} {:?} 24h统计 - 开盘: {} 最高: {} 最低: {} 涨跌: {}%",
                thumb_price.symbol,
                thumb_price.market_type,
                coin_thumb.open,
                coin_thumb.high,
                coin_thumb.low,
                coin_thumb.change
            );
            
            // 更新缓存
            {
                let mut stats_cache = THUMB_STATS_CACHE.lock().unwrap();
                stats_cache.insert(cache_key, (now, coin_thumb));
            }
        }
        Err(e) => {
            log::error!("[ThumbStats] 查询24小时K线数据失败: {}", e);
        }
    }
}

/// 查询24小时内的K线数据（从Redis和MongoDB聚合）
async fn query_24h_klines(
    thumb_price: &CoinThumbPrice,
    start_time: i64,
    end_time: i64,
) -> Result<Vec<Kline>, String> {
    let mut all_klines = Vec::new();
    
    // 使用1分钟K线进行统计（最细粒度）
    let interval = KlineInterval::Min1;
    
    // 1. 从Redis获取数据
    let redis_key = format!(
        "kline:{:?}:{}:{}",
        thumb_price.market_type,
        thumb_price.symbol,
        interval.interval()
    );
    
    if let Ok(redis_data) = cache.hgetall(&redis_key).await {
        for (open_time_str, json_str) in redis_data {
            if let Ok(open_time) = open_time_str.parse::<i64>() {
                // 过滤时间范围
                if open_time >= start_time && open_time <= end_time {
                    if let Ok(kline) = serde_json::from_str::<Kline>(&json_str) {
                        all_klines.push(kline);
                    }
                }
            }
        }
    }
    
    // 2. 从MongoDB获取历史数据
    match Kline::find_many(
        &thumb_price.symbol,
        thumb_price.market_type.clone(),
        interval,
        Some(start_time),
        Some(end_time),
        None, // 不限制数量
    ).await {
        Ok(mongodb_klines) => {
            // 合并数据，去重（以open_time为准，Redis数据优先）
            let mut redis_open_times: std::collections::HashSet<i64> = 
                all_klines.iter().map(|k| k.open_time).collect();
            
            for kline in mongodb_klines {
                if !redis_open_times.contains(&kline.open_time) {
                    all_klines.push(kline);
                }
            }
        }
        Err(e) => {
            log::warn!("[ThumbStats] MongoDB查询失败: {}", e);
        }
    }
    
    // 按时间排序
    all_klines.sort_by_key(|k| k.open_time);
    
    Ok(all_klines)
}

/// 计算行情统计数据
fn calculate_thumb_stats(klines: &[Kline], current_price: Decimal, symbol: &str, timestamp: i64) -> CoinThumb {
    if klines.is_empty() {
        // 如果没有数据，使用当前价格作为默认值
        return CoinThumb::new(
            symbol.to_string(),
            current_price,
            current_price,
            current_price,
            current_price,
            Decimal::ZERO,  // chg
            Decimal::ZERO,  // change
            Decimal::ZERO,  // volume
            Decimal::ZERO,  // quote_volume
        );
    }
    
    // 找到最早的开盘价（24小时开盘价）
    let open = klines.first().map(|k| k.open).unwrap_or(current_price);
    
    // 找到最高价和最低价
    let mut high = current_price;
    let mut low = current_price;
    
    for kline in klines {
        if kline.high > high {
            high = kline.high;
        }
        if kline.low < low {
            low = kline.low;
        }
    }
    
    // 累加成交量和成交额
    let mut volume = Decimal::ZERO;
    let mut quote_volume = Decimal::ZERO;
    
    for kline in klines {
        if let Some(vol) = kline.volume {
            volume += vol;
        }
        if let Some(quote_vol) = kline.quote_volume {
            quote_volume += quote_vol;
        }
    }
    
    // 计算价格变化量（chg）
    let chg = current_price - open;
    
    // 计算涨跌幅（百分比，change）
    let change = if open > Decimal::ZERO {
        (chg / open) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };
    
    let mut coin_thumb = CoinThumb::new(
        symbol.to_string(),
        open,
        high,
        low,
        current_price,  // close
        chg,
        change,
        volume,
        quote_volume,
    );
    
    // 设置时间戳
    coin_thumb.ts = timestamp;
    
    coin_thumb
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn it_works() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        println!("{}", now)
    }
}