// Bitget 交易所接入模块

pub mod spot;
pub mod futures;
mod common;

pub use spot::BitgetSpot;
pub use futures::BitgetFutures;
