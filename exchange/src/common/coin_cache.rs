use dashmap::DashMap;
use once_cell::sync::OnceCell;
use orm::entities::exchange::{AppExchangeSpotCoin, AppExchangeFuturesCoin};
use common::get_db;

/// 现货交易对缓存 Map
/// Key: symbol (如 "BTC/USDT")
static SPOT_COIN_CACHE: OnceCell<DashMap<String, AppExchangeSpotCoin>> = OnceCell::new();

/// 永续合约交易对缓存 Map
/// Key: symbol (如 "BTCUSDT")
static FUTURES_COIN_CACHE: OnceCell<DashMap<String, AppExchangeFuturesCoin>> = OnceCell::new();

/// 初始化现货交易对缓存
pub async fn init_spot_coin_cache() {
    log::info!("正在加载现货交易对配置到缓存...");

    let rb = get_db();
    let spot_coins = match AppExchangeSpotCoin::select_all(rb).await {
        Ok(coins) => coins,
        Err(e) => {
            log::error!("加载现货交易对配置失败: {}", e);
            return;
        }
    };

    let cache = DashMap::new();
    for coin in spot_coins {
        // 优先使用 base_symbol + quote_symbol 作为 key，如果不存在则使用 symbol（去掉 "/"）
        let key = if let (Some(ref base), Some(ref quote)) = (coin.base_symbol.as_ref(), coin.quote_symbol.as_ref()) {
            format!("{}{}", base, quote)
        } else if let Some(ref symbol) = coin.symbol {
            symbol.replace("/", "")
        } else {
            continue; // 跳过无效的交易对
        };
        cache.insert(key, coin);
    }

    if let Err(_) = SPOT_COIN_CACHE.set(cache) {
        log::warn!("现货交易对缓存已初始化，跳过重复初始化");
        return;
    }

    log::info!("✅ 现货交易对缓存加载完成，共 {} 个交易对", SPOT_COIN_CACHE.get().unwrap().len());
}

/// 初始化永续合约交易对缓存
pub async fn init_futures_coin_cache() {
    log::info!("正在加载永续合约交易对配置到缓存...");

    let rb = get_db();
    let futures_coins = match AppExchangeFuturesCoin::select_enabled(rb).await {
        Ok(coins) => coins,
        Err(e) => {
            log::error!("加载永续合约交易对配置失败: {}", e);
            return;
        }
    };

    let cache = DashMap::new();
    for coin in futures_coins {
        // 优先使用 base_asset + quote_asset 作为 key，如果不存在则使用 symbol（去掉 "/"）
        let key = if !coin.base_asset.is_empty() && !coin.quote_asset.is_empty() {
            format!("{}{}", coin.base_asset, coin.quote_asset)
        } else if !coin.symbol.is_empty() {
            coin.symbol.replace("/", "")
        } else {
            continue; // 跳过无效的交易对
        };
        cache.insert(key, coin);
    }

    if let Err(_) = FUTURES_COIN_CACHE.set(cache) {
        log::warn!("永续合约交易对缓存已初始化，跳过重复初始化");
        return;
    }

    log::info!("✅ 永续合约交易对缓存加载完成，共 {} 个交易对", FUTURES_COIN_CACHE.get().unwrap().len());
}

/// 初始化所有缓存
pub fn init_all_caches() {
    tokio::spawn(async move {
        init_spot_coin_cache().await;
        init_futures_coin_cache().await;
    });
}

/// 根据 symbol 获取现货交易对配置
pub fn get_spot_coin(symbol: &str) -> Option<AppExchangeSpotCoin> {
    SPOT_COIN_CACHE.get()
        .and_then(|cache| cache.get(symbol).map(|entry| entry.value().clone()))
}

/// 根据 symbol 获取永续合约交易对配置
pub fn get_futures_coin(symbol: &str) -> Option<AppExchangeFuturesCoin> {
    FUTURES_COIN_CACHE.get()
        .and_then(|cache| cache.get(symbol).map(|entry| entry.value().clone()))
}


/// 获取所有现货交易对
pub fn get_all_spot_coins() -> Vec<AppExchangeSpotCoin> {
    SPOT_COIN_CACHE.get()
        .map(|cache| cache.iter().map(|entry| entry.value().clone()).collect())
        .unwrap_or_default()
}

/// 获取所有永续合约交易对
pub fn get_all_futures_coins() -> Vec<AppExchangeFuturesCoin> {
    FUTURES_COIN_CACHE.get()
        .map(|cache| cache.iter().map(|entry| entry.value().clone()).collect())
        .unwrap_or_default()
}

/// 检查现货交易对是否存在
pub fn spot_coin_exists(symbol: &str) -> bool {
    SPOT_COIN_CACHE.get()
        .map(|cache| cache.contains_key(symbol))
        .unwrap_or(false)
}

/// 检查永续合约交易对是否存在
pub fn futures_coin_exists(symbol: &str) -> bool {
    FUTURES_COIN_CACHE.get()
        .map(|cache| cache.contains_key(symbol))
        .unwrap_or(false)
}

