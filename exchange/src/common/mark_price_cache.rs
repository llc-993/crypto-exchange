use dashmap::DashMap;
use once_cell::sync::OnceCell;
use common::models::UnifiedMarkPrice;
use common::cache;
use common::redis::keys::mark_price as redis_keys;
use rust_decimal::Decimal;

/// 标记价格缓存 Map
/// Key: (symbol, exchange) 元组
/// Value: UnifiedMarkPrice
static MARK_PRICE_CACHE: OnceCell<DashMap<(String, String), UnifiedMarkPrice>> = OnceCell::new();

/// 聚合后的标记价格缓存 Map（按 symbol 聚合，取平均值）
/// Key: symbol
/// Value: UnifiedMarkPrice（funding_rate、index_price 为平均值）
static AGGREGATED_MARK_PRICE_CACHE: OnceCell<DashMap<String, UnifiedMarkPrice>> = OnceCell::new();

/// 初始化标记价格缓存
fn init_mark_price_cache() {
    MARK_PRICE_CACHE.get_or_init(|| DashMap::new());
}

/// 初始化聚合标记价格缓存
fn init_aggregated_mark_price_cache() {
    AGGREGATED_MARK_PRICE_CACHE.get_or_init(|| DashMap::new());
}

/// 初始化所有标记价格缓存
pub fn init_mark_price_caches() {
    init_mark_price_cache();
    init_aggregated_mark_price_cache();
}

/// 初始化所有标记价格缓存并从 Redis 加载数据
pub async fn init_mark_price_caches_from_redis() {
    init_mark_price_caches();
    
    if let Err(e) = load_mark_price_from_redis().await {
        log::warn!("从 Redis 加载标记价格数据失败: {}", e);
    }
    if let Err(e) = load_aggregated_mark_price_from_redis().await {
        log::warn!("从 Redis 加载聚合标记价格数据失败: {}", e);
    }
}

/// 标准化 symbol（去掉 "-" 和 "/"）
fn normalize_symbol(symbol: &str) -> String {
    symbol.replace("-", "").replace("/", "")
}

/// 存储标记价格数据并更新聚合值
pub fn set_mark_price(mark_price: UnifiedMarkPrice) {
    let symbol = normalize_symbol(&mark_price.symbol);
    let exchange = mark_price.exchange.clone();
    
    // 存储原始数据
    if let Some(cache_map) = MARK_PRICE_CACHE.get() {
        cache_map.insert((symbol.clone(), exchange), mark_price.clone());
    }
    
    // 更新聚合数据
    update_aggregated_mark_price(&symbol);
}

/// 更新聚合标记价格（计算平均值）
fn update_aggregated_mark_price(symbol: &str) {
    let Some(raw_cache) = MARK_PRICE_CACHE.get() else { return };
    let Some(agg_cache) = AGGREGATED_MARK_PRICE_CACHE.get() else { return };
    
    // 收集该 symbol 的所有交易所数据
    let mut mark_prices = Vec::new();
    let mut index_prices = Vec::new();
    let mut funding_rates = Vec::new();
    let mut latest_timestamp = 0i64;
    let mut sample_mark_price: Option<UnifiedMarkPrice> = None;
    
    for entry in raw_cache.iter() {
        let (entry_symbol, _exchange) = entry.key();
        if entry_symbol != symbol {
            continue;
        }
        
        let mp = entry.value();
        mark_prices.push(mp.mark_price);
        index_prices.push(mp.index_price);
        
        if let Some(fr) = mp.funding_rate {
            funding_rates.push(fr);
        }
        
        if mp.timestamp > latest_timestamp {
            latest_timestamp = mp.timestamp;
            sample_mark_price = Some(mp.clone());
        }
    }
    
    // 如果没有数据，不更新
    let count = mark_prices.len();
    if count == 0 {
        return;
    }
    
    // 计算平均值
    let avg_mark_price = mark_prices.iter().sum::<Decimal>() / Decimal::from(count);
    let avg_index_price = index_prices.iter().sum::<Decimal>() / Decimal::from(count);
    let avg_funding_rate = if funding_rates.is_empty() {
        None
    } else {
        Some(funding_rates.iter().sum::<Decimal>() / Decimal::from(funding_rates.len()))
    };
    
    // 创建聚合后的 UnifiedMarkPrice
    if let Some(mut aggregated) = sample_mark_price {
        aggregated.exchange = format!("aggregated({})", count); // 标记为聚合数据
        aggregated.symbol = symbol.to_string();
        aggregated.mark_price = avg_mark_price;
        aggregated.index_price = avg_index_price;
        aggregated.funding_rate = avg_funding_rate;
        aggregated.timestamp = latest_timestamp;
        
        agg_cache.insert(symbol.to_string(), aggregated);
        
        #[cfg(debug_assertions)]
        log::debug!("[MarkPrice聚合] {} 平均标记价格: {} 平均指数价格: {} 平均资金费率: {:?} (来自 {} 个交易所)",
            symbol, avg_mark_price, avg_index_price, avg_funding_rate, count);
    }
}

/// 获取原始标记价格
pub fn get_mark_price(symbol: &str, exchange: &str) -> Option<UnifiedMarkPrice> {
    MARK_PRICE_CACHE.get()
        .and_then(|cache_map| {
            let key = (normalize_symbol(symbol), exchange.to_string());
            cache_map.get(&key).map(|entry| entry.value().clone())
        })
}

/// 获取聚合后的标记价格
pub fn get_aggregated_mark_price(symbol: &str) -> Option<UnifiedMarkPrice> {
    AGGREGATED_MARK_PRICE_CACHE.get()
        .and_then(|cache_map| {
            cache_map.get(&normalize_symbol(symbol)).map(|entry| entry.value().clone())
        })
}

/// 获取所有原始标记价格
pub fn get_all_mark_prices() -> Vec<UnifiedMarkPrice> {
    MARK_PRICE_CACHE.get()
        .map(|cache_map| cache_map.iter().map(|entry| entry.value().clone()).collect())
        .unwrap_or_default()
}

/// 获取所有聚合后的标记价格
pub fn get_all_aggregated_mark_prices() -> Vec<UnifiedMarkPrice> {
    AGGREGATED_MARK_PRICE_CACHE.get()
        .map(|cache_map| cache_map.iter().map(|entry| entry.value().clone()).collect())
        .unwrap_or_default()
}

// ==================== Redis 持久化 ====================

/// 从 Redis 加载原始标记价格数据
async fn load_mark_price_from_redis() -> Result<(), Box<dyn std::error::Error>> {
    let data = cache.hgetall(redis_keys::MARK_PRICE).await?;
    
    if data.is_empty() {
        log::info!("[MarkPrice缓存] Redis 中没有原始标记价格数据");
        return Ok(());
    }
    
    let cache_map = MARK_PRICE_CACHE.get()
        .ok_or("标记价格缓存未初始化")?;
    
    let mut count = 0;
    for (field, value) in data {
        // field 格式: "symbol:exchange"
        let parts: Vec<&str> = field.splitn(2, ':').collect();
        if parts.len() != 2 {
            log::warn!("[MarkPrice缓存] 无效的 key: {}", field);
            continue;
        }
        
        match serde_json::from_str::<UnifiedMarkPrice>(&value) {
            Ok(mark_price) => {
                let key = (parts[0].to_string(), parts[1].to_string());
                cache_map.insert(key, mark_price);
                count += 1;
            }
            Err(e) => {
                log::warn!("[MarkPrice缓存] 解析数据失败: {}", e);
            }
        }
    }
    
    log::info!("✅ 从 Redis 加载 {} 条原始标记价格数据", count);
    Ok(())
}

/// 从 Redis 加载聚合标记价格数据
async fn load_aggregated_mark_price_from_redis() -> Result<(), Box<dyn std::error::Error>> {
    let data = cache.hgetall(redis_keys::AGGREGATED_MARK_PRICE).await?;
    
    if data.is_empty() {
        log::info!("[MarkPrice缓存] Redis 中没有聚合标记价格数据");
        return Ok(());
    }
    
    let cache_map = AGGREGATED_MARK_PRICE_CACHE.get()
        .ok_or("聚合标记价格缓存未初始化")?;
    
    let mut count = 0;
    for (field, value) in data {
        match serde_json::from_str::<UnifiedMarkPrice>(&value) {
            Ok(mark_price) => {
                cache_map.insert(field, mark_price);
                count += 1;
            }
            Err(e) => {
                log::warn!("[MarkPrice缓存] 解析聚合数据失败: {}", e);
            }
        }
    }
    
    log::info!("✅ 从 Redis 加载 {} 条聚合标记价格数据", count);
    Ok(())
}

/// 将所有标记价格数据保存到 Redis
pub async fn save_all_mark_prices_to_redis() {
    if let Err(e) = save_mark_price_to_redis().await {
        log::error!("保存原始标记价格数据到 Redis 失败: {}", e);
    }
    if let Err(e) = save_aggregated_mark_price_to_redis().await {
        log::error!("保存聚合标记价格数据到 Redis 失败: {}", e);
    }
}

/// 将原始标记价格数据保存到 Redis
async fn save_mark_price_to_redis() -> Result<(), Box<dyn std::error::Error>> {
    let cache_map = match MARK_PRICE_CACHE.get() {
        Some(c) => c,
        None => {
            log::warn!("[MarkPrice缓存] 原始标记价格缓存未初始化，跳过保存");
            return Ok(());
        }
    };
    
    let mut count = 0;
    for entry in cache_map.iter() {
        let (symbol, exchange) = entry.key();
        let mark_price = entry.value();
        let field = format!("{}:{}", symbol, exchange);
        
        match serde_json::to_string(mark_price) {
            Ok(json) => {
                if let Err(e) = cache.hset(redis_keys::MARK_PRICE, &field, &json).await {
                    log::warn!("[MarkPrice缓存] 保存 {} 到 Redis 失败: {}", field, e);
                } else {
                    count += 1;
                }
            }
            Err(e) => {
                log::warn!("[MarkPrice缓存] 序列化 {} 失败: {}", field, e);
            }
        }
    }
    
    log::info!("✅ 保存 {} 条原始标记价格数据到 Redis", count);
    Ok(())
}

/// 将聚合标记价格数据保存到 Redis
async fn save_aggregated_mark_price_to_redis() -> Result<(), Box<dyn std::error::Error>> {
    let cache_map = match AGGREGATED_MARK_PRICE_CACHE.get() {
        Some(c) => c,
        None => {
            log::warn!("[MarkPrice缓存] 聚合标记价格缓存未初始化，跳过保存");
            return Ok(());
        }
    };
    
    let mut count = 0;
    for entry in cache_map.iter() {
        let symbol = entry.key();
        let mark_price = entry.value();
        
        match serde_json::to_string(mark_price) {
            Ok(json) => {
                if let Err(e) = cache.hset(redis_keys::AGGREGATED_MARK_PRICE, symbol, &json).await {
                    log::warn!("[MarkPrice缓存] 保存聚合 {} 到 Redis 失败: {}", symbol, e);
                } else {
                    count += 1;
                }
            }
            Err(e) => {
                log::warn!("[MarkPrice缓存] 序列化聚合 {} 失败: {}", symbol, e);
            }
        }
    }
    
    log::info!("✅ 保存 {} 条聚合标记价格数据到 Redis", count);
    Ok(())
}
