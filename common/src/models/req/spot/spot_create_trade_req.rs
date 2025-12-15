// 现货下单
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExchangeOrderReq {
    // 方向 0:买入 1:卖出
    pub direction: i16,

    // 交易对
    pub symbol: String,

    // 买入价格
    pub price: Decimal,

    // 买入数量
    pub amount: Decimal,

    // 交易类型 0:市价交易 1:限价交易
    #[serde(rename = "type")]
    pub order_type: i8,
}

