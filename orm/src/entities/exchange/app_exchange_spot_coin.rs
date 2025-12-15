use common::get_db;
use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

fn i8_to_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<i8>::deserialize(deserializer)?;
    Ok(opt.map(|v| v != 0))
}

/// 现货交易对配置表
///
/// 用于配置币币交易对的基本信息、手续费率、交易限制等
///
/// # 表名
/// `app_exchange_spot_coin`
///
/// # 主要字段
/// - `symbol`: 交易对，格式如 BTC/USDT（USDT是计价货币quote，BTC是交易资产base）
/// - `data_source`: 数据源（如 binance, okx, bitget）
/// - `buy_fee_rate`/`sell_fee_rate`: 买卖手续费率
/// - `can_show`/`can_buy`/`can_sell`: 显示和交易控制开关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeSpotCoin {
    /// 主键ID
    pub id: Option<i64>,
    /// 交易对符号，如 BTC/USDT
    pub symbol: Option<String>,
    /// 图标URL
    pub icon: Option<String>,
    /// 数据源：binance, okx, bitget 等
    pub data_source: Option<String>,
    /// 参数配置（JSON格式）
    pub params: Option<String>,
    /// 买方手续费率
    pub buy_fee_rate: Option<Decimal>,
    /// 卖方手续费率
    pub sell_fee_rate: Option<Decimal>,
    /// 交易资产（base），如 BTC
    pub base_symbol: Option<String>,
    /// 计价货币（quote），如 USDT
    pub quote_symbol: Option<String>,
    /// 计价货币精度
    pub quote_scale: i32,
    /// 交易资产精度
    pub base_scale: i32,
    #[serde(deserialize_with = "i8_to_bool")]
    pub can_show: Option<bool>,
    #[serde(deserialize_with = "i8_to_bool")]
    pub can_buy: Option<bool>,
    #[serde(deserialize_with = "i8_to_bool")]
    pub can_sell: Option<bool>,
    #[serde(deserialize_with = "i8_to_bool")]
    pub show_hot: Option<bool>,
    #[serde(deserialize_with = "i8_to_bool")]
    pub show_chg: Option<bool>,
    pub enable_order_robot: i16,
    pub auto_trigger_time: i32,
    pub buyer_low_price_rate: Option<Decimal>,
    pub seller_over_price_rate: Option<Decimal>,
    pub order_expire_time: i32,
    pub detention: i64,
    pub length: i32,
    pub random_price: Option<Decimal>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub robot_quote_scale: Option<i32>,
    pub robot_base_scale: Option<i32>,
    pub market_type: i32,
    pub sort_level: Option<i32>,
    pub order_limit: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppExchangeSpotCoin {}, "app_exchange_spot_coin");
rbatis::impl_select!(AppExchangeSpotCoin {
    select_spot_by_data_source(data_source: String) => "`where data_source = #{data_source}`" }, "app_exchange_spot_coin");

rbatis::impl_select!(AppExchangeSpotCoin{
    select_spot_by_symbol(symbol: String) -> Option  => "`where symbol = #{symbol} `"
}, "app_exchange_spot_coin");

impl AppExchangeSpotCoin {
    pub const TABLE_NAME: &'static str = "app_exchange_spot_coin";

    pub async fn select_spot_coin_by_data_source(
        data_source: String,
    ) -> Result<Vec<Self>, rbdc::Error> {
        let rb = get_db();
        Self::select_spot_by_data_source(rb, data_source).await
    }

    pub async fn select_spot_coin_by_symbol(symbol: String) -> Result<Option<Self>, rbdc::Error> {
        let rb = get_db();
        Self::select_spot_by_symbol(rb, symbol).await
    }
}
