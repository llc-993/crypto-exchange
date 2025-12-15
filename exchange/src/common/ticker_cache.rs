use dashmap::DashMap;
use once_cell::sync::OnceCell;
use common::models::UnifiedTicker;
use common::cache;
use common::redis::keys::ticker as redis_keys;
use rust_decimal::Decimal;
use crate::common::coin_cache;

/// 现货 Ticker 缓存 Map
/// Key: symbol (去掉 "-" 和 "/" 后的符号，如 "BTCUSDT")
/// Value: UnifiedTicker
static SPOT_TICKER_CACHE: OnceCell<DashMap<String, UnifiedTicker>> = OnceCell::new();

/// 永续合约 Ticker 缓存 Map
/// Key: (exchange, symbol) 元组
/// Value: UnifiedTicker
static FUTURES_TICKER_CACHE: OnceCell<DashMap<(String, String), UnifiedTicker>> = OnceCell::new();

/// 初始化现货 Ticker 缓存
fn init_spot_ticker_cache() {
    SPOT_TICKER_CACHE.get_or_init(|| DashMap::new());
}

/// 初始化永续合约 Ticker 缓存
fn init_futures_ticker_cache() {
    FUTURES_TICKER_CACHE.get_or_init(|| DashMap::new());
}

/// 初始化所有 Ticker 缓存
pub fn init_ticker_caches() {
    init_spot_ticker_cache();
    init_futures_ticker_cache();
}

/// 初始化所有 Ticker 缓存并从 Redis 加载数据
pub async fn init_ticker_caches_from_redis() {
    init_spot_ticker_cache();
    init_futures_ticker_cache();
    
    // 从 Redis 加载数据
    if let Err(e) = load_spot_tickers_from_redis().await {
        log::warn!("从 Redis 加载现货 Ticker 数据失败: {}", e);
    }
    if let Err(e) = load_futures_tickers_from_redis().await {
        log::warn!("从 Redis 加载永续 Ticker 数据失败: {}", e);
    }
}

/// 从 Redis 加载现货 Ticker 数据
async fn load_spot_tickers_from_redis() -> Result<(), Box<dyn std::error::Error>> {
    let data = cache.hgetall(redis_keys::SPOT_TICKER).await?;
    
    if data.is_empty() {
        log::info!("[Ticker缓存] Redis 中没有现货 Ticker 数据");
        return Ok(());
    }
    
    let cache_map = SPOT_TICKER_CACHE.get()
        .ok_or("现货 Ticker 缓存未初始化")?;
    
    let mut count = 0;
    for (field, value) in data {
        match serde_json::from_str::<UnifiedTicker>(&value) {
            Ok(ticker) => {
                cache_map.insert(field, ticker);
                count += 1;
            }
            Err(e) => {
                log::warn!("[Ticker缓存] 解析现货 Ticker 数据失败: {}", e);
            }
        }
    }
    
    log::info!("✅ 从 Redis 加载 {} 条现货 Ticker 数据", count);
    Ok(())
}

/// 从 Redis 加载永续 Ticker 数据
async fn load_futures_tickers_from_redis() -> Result<(), Box<dyn std::error::Error>> {
    let data = cache.hgetall(redis_keys::FUTURES_TICKER).await?;
    
    if data.is_empty() {
        log::info!("[Ticker缓存] Redis 中没有永续 Ticker 数据");
        return Ok(());
    }
    
    let cache_map = FUTURES_TICKER_CACHE.get()
        .ok_or("永续 Ticker 缓存未初始化")?;
    
    let mut count = 0;
    for (field, value) in data {
        // field 格式: "exchange:symbol"
        let parts: Vec<&str> = field.splitn(2, ':').collect();
        if parts.len() != 2 {
            log::warn!("[Ticker缓存] 无效的永续 Ticker key: {}", field);
            continue;
        }
        
        match serde_json::from_str::<UnifiedTicker>(&value) {
            Ok(ticker) => {
                let key = (parts[0].to_string(), parts[1].to_string());
                cache_map.insert(key, ticker);
                count += 1;
            }
            Err(e) => {
                log::warn!("[Ticker缓存] 解析永续 Ticker 数据失败: {}", e);
            }
        }
    }
    
    log::info!("✅ 从 Redis 加载 {} 条永续 Ticker 数据", count);
    Ok(())
}

/// 将所有 Ticker 数据保存到 Redis
pub async fn save_all_tickers_to_redis() {
    if let Err(e) = save_spot_tickers_to_redis().await {
        log::error!("保存现货 Ticker 数据到 Redis 失败: {}", e);
    }
    if let Err(e) = save_futures_tickers_to_redis().await {
        log::error!("保存永续 Ticker 数据到 Redis 失败: {}", e);
    }
}

/// 将现货 Ticker 数据保存到 Redis
async fn save_spot_tickers_to_redis() -> Result<(), Box<dyn std::error::Error>> {
    let cache_map = match SPOT_TICKER_CACHE.get() {
        Some(c) => c,
        None => {
            log::warn!("[Ticker缓存] 现货 Ticker 缓存未初始化，跳过保存");
            return Ok(());
        }
    };
    
    let mut count = 0;
    for entry in cache_map.iter() {
        let field = entry.key();
        let ticker = entry.value();
        
        match serde_json::to_string(ticker) {
            Ok(json) => {
                if let Err(e) = cache.hset(redis_keys::SPOT_TICKER, field, &json).await {
                    log::warn!("[Ticker缓存] 保存现货 Ticker {} 到 Redis 失败: {}", field, e);
                } else {
                    count += 1;
                }
            }
            Err(e) => {
                log::warn!("[Ticker缓存] 序列化现货 Ticker {} 失败: {}", field, e);
            }
        }
    }
    
    log::info!("✅ 保存 {} 条现货 Ticker 数据到 Redis", count);
    Ok(())
}

/// 将永续 Ticker 数据保存到 Redis
async fn save_futures_tickers_to_redis() -> Result<(), Box<dyn std::error::Error>> {
    let cache_map = match FUTURES_TICKER_CACHE.get() {
        Some(c) => c,
        None => {
            log::warn!("[Ticker缓存] 永续 Ticker 缓存未初始化，跳过保存");
            return Ok(());
        }
    };
    
    let mut count = 0;
    for entry in cache_map.iter() {
        let (exchange, symbol) = entry.key();
        let ticker = entry.value();
        let field = format!("{}:{}", exchange, symbol);
        
        match serde_json::to_string(ticker) {
            Ok(json) => {
                if let Err(e) = cache.hset(redis_keys::FUTURES_TICKER, &field, &json).await {
                    log::warn!("[Ticker缓存] 保存永续 Ticker {} 到 Redis 失败: {}", field, e);
                } else {
                    count += 1;
                }
            }
            Err(e) => {
                log::warn!("[Ticker缓存] 序列化永续 Ticker {} 失败: {}", field, e);
            }
        }
    }
    
    log::info!("✅ 保存 {} 条永续 Ticker 数据到 Redis", count);
    Ok(())
}

/// 标准化 symbol（去掉 "-" 和 "/"）
fn normalize_symbol(symbol: &str) -> String {
    symbol.replace("-", "").replace("/", "")
}

/// 存储现货 Ticker 数据
pub fn set_spot_ticker(ticker: UnifiedTicker) {
    if let Some(ticker_map) = SPOT_TICKER_CACHE.get() {
        let key = normalize_symbol(&ticker.symbol);
        ticker_map.insert(key, ticker);
    }
}

/// 存储永续合约 Ticker 数据
pub fn set_futures_ticker(ticker: UnifiedTicker) {
    if let Some(ticker_map) = FUTURES_TICKER_CACHE.get() {
        let key = (ticker.exchange.clone(), ticker.symbol.clone());
        ticker_map.insert(key, ticker);
    }
}

/// 根据 symbol 获取现货 Ticker
pub fn get_spot_ticker(symbol: &str) -> Option<UnifiedTicker> {
    SPOT_TICKER_CACHE.get()
        .and_then(|ticker_map| {
            let key = normalize_symbol(symbol);
            ticker_map.get(&key)
                .map(|entry| entry.value().clone())
        })
}

/// 根据 exchange 和 symbol 获取永续合约 Ticker
pub fn get_futures_ticker(exchange: &str, symbol: &str) -> Option<UnifiedTicker> {
    FUTURES_TICKER_CACHE.get()
        .and_then(|ticker_map| {
            ticker_map.get(&(exchange.to_string(), symbol.to_string()))
                .map(|entry| entry.value().clone())
        })
}

/// 获取所有现货 Ticker
pub fn get_all_spot_tickers() -> Vec<UnifiedTicker> {
    SPOT_TICKER_CACHE.get()
        .map(|ticker_map| ticker_map.iter().map(|entry| entry.value().clone()).collect())
        .unwrap_or_default()
}

/// 获取所有永续合约 Ticker
pub fn get_all_futures_tickers() -> Vec<UnifiedTicker> {
    FUTURES_TICKER_CACHE.get()
        .map(|ticker_map| ticker_map.iter().map(|entry| entry.value().clone()).collect())
        .unwrap_or_default()
}

/// 检查现货 Ticker 是否存在
pub fn spot_ticker_exists(symbol: &str) -> bool {
    SPOT_TICKER_CACHE.get()
        .map(|ticker_map| {
            let key = normalize_symbol(symbol);
            ticker_map.contains_key(&key)
        })
        .unwrap_or(false)
}

/// 检查永续合约 Ticker 是否存在
pub fn futures_ticker_exists(exchange: &str, symbol: &str) -> bool {
    FUTURES_TICKER_CACHE.get()
        .map(|ticker_map| ticker_map.contains_key(&(exchange.to_string(), symbol.to_string())))
        .unwrap_or(false)
}

/// 根据 market_type 自动存储 Ticker（推荐使用）
pub fn set_ticker(ticker: UnifiedTicker) {
    match ticker.market_type {
        common::models::MarketType::Spot => set_spot_ticker(ticker),
        common::models::MarketType::Futures => set_futures_ticker(ticker),
    }
}

/// 计算指定 symbol 的 Futures Ticker 加权平均价格
/// 使用 AppExchangeFuturesCoin::index_price_source 中的权重配置
/// 返回 (加权平均价格, 参与计算的交易所数量)
pub fn calculate_weighted_futures_price(symbol: &str) -> Option<(Decimal, usize)> {
    // 获取缓存和交易对配置
    let ticker_map = FUTURES_TICKER_CACHE.get()?;
    let futures_coin = coin_cache::get_futures_coin(symbol)?;
    let index_price_source = futures_coin.index_price_source.as_object()?;
    
    let normalized_symbol = normalize_symbol(symbol);
    let mut total_weighted_price = Decimal::ZERO;
    let mut total_weight = Decimal::ZERO;
    let mut count = 0;
    
    // 遍历所有 Futures Ticker，计算加权平均价格
    for entry in ticker_map.iter() {
        let ticker = entry.value();
        
        // 检查 symbol 和 market_type 是否匹配
        if normalize_symbol(&ticker.symbol) != normalized_symbol 
            || ticker.market_type != common::models::MarketType::Futures {
            continue;
        }
        
        // 从配置中获取该交易所的权重
        let weight = match get_exchange_weight(index_price_source, &ticker.exchange) {
            Some(w) if w > Decimal::ZERO => w,
            _ => {
                log::warn!("[加权计算] 交易所 {} 未在 index_price_source 配置中找到或权重为0，跳过", ticker.exchange);
                continue;
            }
        };
        
        // 累加加权价格
        total_weighted_price += ticker.close * weight;
        total_weight += weight;
        count += 1;
        
        #[cfg(debug_assertions)]
        {
            log::debug!("[加权计算] 交易所: {} 价格: {} 权重: {}", 
                ticker.exchange, ticker.close, weight);
        }
    }
    
    // 返回加权平均价格
    if count > 0 && total_weight > Decimal::ZERO {
        let weighted_price = total_weighted_price / total_weight;
        log::debug!("[加权计算] {} 最终加权价格: {} (来自 {} 个交易所)", 
            symbol, weighted_price, count);
        Some((weighted_price, count))
    } else {
        None
    }
}

/// 从 index_price_source 配置中获取交易所的权重
fn get_exchange_weight(index_price_source: &serde_json::Map<String, serde_json::Value>, exchange: &str) -> Option<Decimal> {
    index_price_source
        .get(exchange)?
        .get("weight")?
        .as_f64()
        .and_then(|w| Decimal::try_from(w).ok())
}

