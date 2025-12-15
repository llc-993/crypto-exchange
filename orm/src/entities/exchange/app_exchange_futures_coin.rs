use rbatis::crud;
use rbatis::rbdc::datetime::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 主流永续合约配置表（Binance/Bitget/OKX）
/// 
/// 用于配置主流交易所的永续合约交易对信息、杠杆倍率、手续费、资金费率等
/// 
/// # 表名
/// `app_exchange_futures_coin`
/// 
/// # 主要字段
/// - `symbol`: 交易对，如 BTCUSDT
/// - `base_asset`: 基础资产，如 BTC
/// - `quote_asset`: 计价资产，如 USDT
/// - `settle_asset`: 结算资产，USDT-M 永续则为 USDT
/// - `contract_type`: 合约类型（perpetual 永续）
/// - `max_leverage`: 最大杠杆倍数
/// - `funding_interval`: 资金费率结算周期（小时）
/// - `supported_exchanges`: 支持的交易所列表（binance,bitget,okx）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExchangeFuturesCoin {
    /// 主键ID
    pub id: Option<i64>,
    /// 交易对符号，例如 BTCUSDT
    pub symbol: String,
    /// 基础资产，如 BTC
    pub base_asset: String,
    /// 计价资产，如 USDT
    pub quote_asset: String,
    /// 结算资产，USDT-M 永续则为 USDT
    pub settle_asset: String,
    /// 合约类型：perpetual(永续)
    pub contract_type: String,
    /// 保证金模式：isolated(逐仓) 或 cross(全仓)
    pub margin_type: String,
    /// 价格小数位精度
    pub price_precision: i32,
    /// 数量小数位精度
    pub quantity_precision: i32,
    /// 最小下单数量
    pub min_order_qty: Decimal,
    /// 最小名义价值（USDT）
    pub min_order_value: Decimal,
    /// 最大杠杆倍数
    pub max_leverage: i32,
    /// 杠杆档位 JSON 配置
    /// 
    /// 示例格式：
    /// ```json
    /// [
    ///   {"bracket": 1, "max_notional": 50000, "max_leverage": 125},
    ///   {"bracket": 2, "max_notional": 250000, "max_leverage": 100}
    /// ]
    /// ```
    pub leverage_brackets: JsonValue,
    /// 挂单手续费率（Maker Fee）
    pub maker_fee: Decimal,
    /// 吃单手续费率（Taker Fee）
    pub taker_fee: Decimal,
    /// 资金费率结算周期（小时）
    pub funding_interval: i32,
    /// 资金费率上限
    pub funding_rate_cap: Decimal,
    /// 资金费率下限
    pub funding_rate_floor: Decimal,
    /// 资金费率结算时间，如 "00:00,08:00,16:00"
    pub funding_settlement_time: String,
    /// 指数价格来源 JSON 配置（仅 binance、bitget、okx）
    /// 
    /// 示例格式：
    /// ```json
    /// {
    ///   "binance": {"weight": 0.4},
    ///   "bitget": {"weight": 0.3},
    ///   "okx": {"weight": 0.3}
    /// }
    /// ```
    pub index_price_source: JsonValue,
    /// 标记价格机制配置 JSON
    /// 
    /// 示例格式：
    /// ```json
    /// {
    ///   "method": "fair_price",
    ///   "ema_period": 300
    /// }
    /// ```
    pub mark_price_config: JsonValue,
    /// 是否启用：1-启用，0-禁用
    pub status: i8,
    /// 支持的交易所列表，如 "binance,bitget,okx"
    pub supported_exchanges: String,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

crud!(AppExchangeFuturesCoin {}, "app_exchange_futures_coin");

rbatis::impl_select!(AppExchangeFuturesCoin {
    select_by_symbol(symbol: String) => "`where symbol = #{symbol}`" 
}, "app_exchange_futures_coin");

rbatis::impl_select!(AppExchangeFuturesCoin {
    select_enabled() => "`where status = 1`" 
}, "app_exchange_futures_coin");

rbatis::impl_select!(AppExchangeFuturesCoin {
    select_by_exchange_internal(exchange: String) => "`where status = 1 and FIND_IN_SET(#{exchange}, supported_exchanges) > 0`" 
}, "app_exchange_futures_coin");

impl AppExchangeFuturesCoin {
    pub const TABLE_NAME: &'static str = "app_exchange_futures_coin";
    
    /// 根据交易所查询启用的永续合约配置
    /// 
    /// # 参数
    /// - `exchange`: 交易所名称（如 "binance", "bitget", "okx"）
    /// 
    /// # 返回
    /// 返回该交易所支持的所有启用的永续合约配置列表
    /// 
    /// # 示例
    /// ```rust
    /// let binance_futures = AppExchangeFuturesCoin::select_futures_coin_by_exchange("binance".to_string()).await?;
    /// ```
    pub async fn select_futures_coin_by_exchange(exchange: String) -> Result<Vec<Self>, rbdc::Error> {
        let rb = common::get_db();
        Self::select_by_exchange_internal(rb, exchange).await
    }
    
    /// 检查是否支持指定交易所
    pub fn supports_exchange(&self, exchange: &str) -> bool {
        self.supported_exchanges
            .split(',')
            .any(|e| e.trim().eq_ignore_ascii_case(exchange))
    }
    
    /// 获取支持的交易所列表
    pub fn get_supported_exchanges(&self) -> Vec<String> {
        self.supported_exchanges
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }
}
