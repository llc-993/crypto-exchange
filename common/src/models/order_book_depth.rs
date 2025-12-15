use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug, Clone)]
pub struct Level {
    pub price: Decimal,
    pub amount: Decimal,
}

#[derive(Serialize, Deserialize,Debug, Clone)]
pub struct OrderBookDepth {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}


