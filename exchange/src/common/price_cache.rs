use once_cell::sync::OnceCell;
use dashmap::DashMap;
use rust_decimal::Decimal;
use common::MarketType;

static SPOT_PRICE_CACHE: OnceCell<DashMap<String, Decimal>> = OnceCell::new();
static FUTURES_PRICE_CACHE: OnceCell<DashMap<String, Decimal>> = OnceCell::new();

pub fn init_price_cache() {
    SPOT_PRICE_CACHE.get_or_init(DashMap::new);
    FUTURES_PRICE_CACHE.get_or_init(DashMap::new);
}

pub fn set_final_price(symbol: &str, price: Decimal, market_type: &MarketType) {
    match market_type {
        MarketType::Spot => {
            if let Some(cache) = SPOT_PRICE_CACHE.get() {
                cache.insert(symbol.to_string(), price);
            }
        }
        MarketType::Futures => {
            if let Some(cache) = FUTURES_PRICE_CACHE.get() {
                cache.insert(symbol.to_string(), price);
            }
        }
    }
}

pub fn get_price(symbol: &str, market_type: &MarketType) -> Option<Decimal> {
    match market_type {
        MarketType::Spot => SPOT_PRICE_CACHE.get()?.get(symbol).map(|v| *v),
        MarketType::Futures => FUTURES_PRICE_CACHE.get()?.get(symbol).map(|v| *v),
    }
}