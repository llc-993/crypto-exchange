// Bitget 永续合约交易接入模块
use orm::entities::exchange::AppExchangeFuturesCoin;
use common::PulsarClient;
use std::sync::Arc;
use super::common::{BitgetWebSocket, BitgetInstType, BitgetSymbol};

// 为 AppExchangeFuturesCoin 实现 BitgetSymbol trait
impl BitgetSymbol for AppExchangeFuturesCoin {
    fn get_symbol(&self) -> Option<&String> {
        Some(&self.symbol)
    }
}

/// Bitget 永续合约接入
pub struct BitgetFutures {
    ws_client: BitgetWebSocket<AppExchangeFuturesCoin>,
}

impl BitgetFutures {
    pub fn new() -> Self {
        Self {
            ws_client: BitgetWebSocket::new(BitgetInstType::UsdtFutures),
        }
    }

    /// 设置 PulsarClient
    pub fn with_pulsar(mut self, pulsar_client: Arc<PulsarClient>) -> Self {
        self.ws_client = self.ws_client.with_pulsar(pulsar_client);
        self
    }

    /// 加载永续合约配置
    pub async fn load_futures_coins(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("正在加载 Bitget 永续合约配置...");
        
        match AppExchangeFuturesCoin::select_futures_coin_by_exchange("bitget".to_string()).await {
            Ok(coin_list) => {
                let count = coin_list.len();
                self.ws_client.set_symbols(coin_list).await;
                log::info!("✅ Bitget 永续合约配置加载完成，共 {} 个合约", count);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Bitget 永续合约配置加载失败: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// 获取永续合约数量
    pub async fn get_futures_coin_count(&self) -> usize {
        self.ws_client.get_symbol_count().await
    }

    /// 启动 WebSocket 数据接入
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Bitget 永续合约数据接入服务启动中...");
        self.ws_client.start().await
    }
}

impl Default for BitgetFutures {
    fn default() -> Self {
        Self::new()
    }
}
