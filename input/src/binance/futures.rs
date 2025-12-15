// Binance 永续合约交易接入模块
use orm::entities::exchange::AppExchangeFuturesCoin;
use common::PulsarClient;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{StreamExt, SinkExt};
use serde_json::json;
use common::pulsar::topics;

/// WebSocket 重连配置
#[derive(Clone)]
struct ReconnectConfig {
    initial_delay: Duration,
    max_delay: Duration,
    backoff_factor: u32,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2,
            heartbeat_interval: Duration::from_secs(180), // Binance: 3分钟
            heartbeat_timeout: Duration::from_secs(600),  // 10分钟
        }
    }
}

/// 计算指数退避延迟
fn calculate_backoff_delay(retry_count: u32, config: &ReconnectConfig) -> Duration {
    let delay_secs = config.initial_delay.as_secs() 
        * (config.backoff_factor.pow(retry_count.min(6)) as u64);
    Duration::from_secs(delay_secs.min(config.max_delay.as_secs()))
}

/// Binance 永续合约接入
pub struct BinanceFutures {
    futures_coins: Arc<RwLock<Vec<AppExchangeFuturesCoin>>>,
    pulsar_client: Option<Arc<PulsarClient>>,
}

impl BinanceFutures {
    pub fn new() -> Self {
        Self {
            futures_coins: Arc::new(RwLock::new(Vec::new())),
            pulsar_client: None,
        }
    }

    /// 设置 PulsarClient
    pub fn with_pulsar(mut self, pulsar_client: Arc<PulsarClient>) -> Self {
        self.pulsar_client = Some(pulsar_client);
        self
    }

    /// 加载永续合约配置
    pub async fn load_futures_coins(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("正在加载 Binance 永续合约配置...");
        
        match AppExchangeFuturesCoin::select_futures_coin_by_exchange("binance".to_string()).await {
            Ok(coin_list) => {
                let count = coin_list.len();
                let mut futures_coins = self.futures_coins.write().await;
                *futures_coins = coin_list;
                log::info!("✅ Binance 永续合约配置加载完成，共 {} 个合约", count);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Binance 永续合约配置加载失败: {}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn get_futures_coin_count(&self) -> usize {
        self.futures_coins.read().await.len()
    }

    pub async fn get_futures_coins(&self) -> Vec<AppExchangeFuturesCoin> {
        self.futures_coins.read().await.clone()
    }

    /// 启动 WebSocket 数据接入
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Binance 永续合约数据接入服务启动中...");
        
        let futures_coins = self.get_futures_coins().await;
        if futures_coins.is_empty() {
            log::warn!("没有加载到 Binance 永续合约配置，跳过数据接入");
            return Ok(());
        }
        
        log::info!("开始订阅 {} 个永续合约的实时数据", futures_coins.len());
        
        let pulsar_client = self.pulsar_client.clone();
        tokio::spawn(async move {
            Self::run_websocket_loop(futures_coins, pulsar_client).await;
        });
        
        log::info!("✅ Binance 永续合约数据订阅任务已启动");
        Ok(())
    }

    /// 运行 WebSocket 连接循环（带自动重连）
    async fn run_websocket_loop(futures_coins: Vec<AppExchangeFuturesCoin>, pulsar_client: Option<Arc<PulsarClient>>) {
        
        
        let config = ReconnectConfig::default();
        let mut retry_count = 0;
        
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);
        
        'reconnect: loop {
            log::info!("🔌 正在连接到 Binance Futures WebSocket...");
            
            // Binance Futures 组合流 URL
            let url = "wss://fstream.binance.com/stream";
            let (ws_stream, _) = match connect_async(url).await {
                Ok(stream) => {
                    log::info!("✅ Binance Futures WebSocket 连接成功");
                    retry_count = 0;
                    stream
                }
                Err(e) => {
                    log::error!("❌ 连接失败: {}", e);
                    let delay = calculate_backoff_delay(retry_count, &config);
                    log::warn!("⏳ {}秒后重新连接...", delay.as_secs());
                    
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        _ = &mut ctrl_c => {
                            log::info!("收到关闭信号");
                            break 'reconnect;
                        }
                    }
                }
            };
            
            let (mut write, mut read) = ws_stream.split();
            
            // 构建订阅流列表
            let mut streams = Vec::new();
            for coin in &futures_coins {
                let symbol = coin.symbol.to_lowercase();
                
                // 1. Ticker Stream
                streams.push(format!("{}@ticker", symbol));
                
                // 2. Mark Price Stream (1秒更新)
                streams.push(format!("{}@markPrice@1s", symbol));
                
                // 3. K线 Streams (8个时间间隔)
                /*for interval in KlineInterval::all() {
                    let binance_interval = interval.binance_interval();
                    streams.push(format!("{}@kline_{}", symbol, binance_interval));
                }
                
                // 4. Depth Stream (20档，100ms更新)
                streams.push(format!("{}@depth20@100ms", symbol));*/
            }
            
            log::info!("准备订阅 {} 个数据流", streams.len());
            
            // 分批订阅，避免超过限制
            const BATCH_SIZE: usize = 100;
            let batches: Vec<_> = streams.chunks(BATCH_SIZE).collect();
            
            log::info!("分 {} 批订阅，每批最多 {} 个流", batches.len(), BATCH_SIZE);
            
            for (i, batch) in batches.iter().enumerate() {
                let subscribe_msg = json!({
                    "method": "SUBSCRIBE",
                    "params": batch,
                    "id": i + 1
                });
                
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    log::error!("发送第 {} 批订阅失败: {}", i + 1, e);
                    retry_count += 1;
                    continue 'reconnect;
                }
                log::info!("✅ 第 {}/{} 批订阅已发送（{} 个流）", i + 1, batches.len(), batch.len());
                
                // 批次间延迟，避免超过每秒5个请求限制
                if i < batches.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            }
            
            log::info!("✅ 所有订阅请求已发送");
            
            // 消息处理循环
            let mut message_count = 0;
            let mut last_log_time = Instant::now();
            let mut last_message_time = Instant::now();
            let mut heartbeat_timer = interval(config.heartbeat_interval);
            heartbeat_timer.tick().await;
            
            loop {
                tokio::select! {
                    _ = heartbeat_timer.tick() => {
                        if last_message_time.elapsed() > config.heartbeat_timeout {
                            log::warn!("💔 心跳超时，主动断开重连");
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        
                        // Binance 使用标准 WebSocket Ping
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            log::error!("发送心跳失败: {}", e);
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        log::debug!("💓 发送心跳 ping");
                    }
                    
                    msg = read.next() => {
                        let msg = match msg {
                            Some(m) => m,
                            None => {
                                log::warn!("WebSocket 流已结束，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                        };
                        
                        last_message_time = Instant::now();
                        
                        match msg {
                            Ok(Message::Text(text)) => {
                                message_count += 1;
                                
                                if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                    log::debug!("收到消息 #{}: {}", message_count,
                                        if text.len() > 200 { &text[..200] } else { &text });
                                    last_log_time = Instant::now();
                                }
                                
                                // 解析 Binance Futures 消息
                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(stream) = json_msg.get("stream").and_then(|v| v.as_str()) {
                                        if let Some(data) = json_msg.get("data") {
                                            Self::handle_stream_data(stream, data, pulsar_client.clone());
                                        }
                                    } else if let Some(result) = json_msg.get("result") {
                                        if result.is_null() {
                                            log::info!("📩 订阅确认: id={}", json_msg.get("id").unwrap_or(&json!(0)));
                                        }
                                    } else if let Some(error) = json_msg.get("error") {
                                        log::error!("❌ 订阅错误: {}", error);
                                    }
                                }
                                
                                if message_count % 1000 == 0 {
                                    log::debug!("已接收 {} 条消息", message_count);
                                }
                            }
                            Ok(Message::Ping(payload)) => {
                                if let Err(e) = write.send(Message::Pong(payload)).await {
                                    log::error!("回复 Pong 失败: {}", e);
                                    retry_count += 1;
                                    continue 'reconnect;
                                }
                                log::debug!("收到 Ping，已回复 Pong");
                            }
                            Ok(Message::Pong(_)) => {
                                log::debug!("💓 收到 Pong");
                            }
                            Ok(Message::Close(_)) => {
                                log::warn!("收到 Close 帧，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            Err(e) => {
                                log::error!("WebSocket 错误: {}, 准备重连", e);
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            _ => {}
                        }
                    }
                    
                    _ = &mut ctrl_c => {
                        log::info!("收到关闭信号，停止 WebSocket");
                        break 'reconnect;
                    }
                }
            }
        }
        
        log::info!("Binance Futures WebSocket 守护任务已停止");
    }

    /// 处理不同类型的数据流
    fn handle_stream_data(stream: &str, data: &serde_json::Value, pulsar_client: Option<Arc<PulsarClient>>) {
        if stream.contains("@ticker") {
            Self::handle_ticker_data(stream, data, pulsar_client);
        } else if stream.contains("@kline_") {
            Self::handle_kline_data(stream, data);
        } else if stream.contains("@markPrice") {
            Self::handle_mark_price_data(stream, data);
        } else if stream.contains("@depth") {
            Self::handle_depth_data(stream, data);
        }
    }

    /// 处理 Ticker 数据
    fn handle_ticker_data(_stream: &str, data: &serde_json::Value, _pulsar_client: Option<Arc<PulsarClient>>) {
        let symbol = data.get("s").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        log::debug!("[Binance Futures {}] Ticker - 原始数据: {:?}", symbol, data);

        // 转换为 UnifiedTicker 并发送到 Pulsar
        match common::TickerConverter::from_binance_futures(data, symbol) {
            Ok(unified_ticker) => {
                log::debug!(
                    "[Binance Futures {}] 转换成功 - 价格: {}, 涨跌幅: {:?}%",
                    symbol, unified_ticker.close, unified_ticker.change_percent_24h
                );
                common::PulsarClient::publish_async(topics::ticker::FUTURES_TICKER, unified_ticker);
            }
            Err(e) => log::error!("[Binance Futures {}] Ticker 转换失败: {}", symbol, e),
        }
    }

    /// 处理 K线数据
    fn handle_kline_data(_stream: &str, data: &serde_json::Value) {
        if let Some(k) = data.get("k") {
            let symbol = k.get("s").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
            let interval = k.get("i").and_then(|v| v.as_str()).unwrap_or("?");
            let close = k.get("c").and_then(|v| v.as_str()).unwrap_or("0");
            let is_closed = k.get("x").and_then(|v| v.as_bool()).unwrap_or(false);
            
            if is_closed {
                log::debug!("[{}] Kline {} - 收盘价: {}", symbol, interval, close);
            }
        }
    }

    /// 处理标记价格数据
    fn handle_mark_price_data(_stream: &str, data: &serde_json::Value) {
        let symbol = data.get("s").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        
        log::debug!("[Binance Futures {}] Mark Price - 原始数据: {:?}", symbol, data);

        // 转换为 UnifiedMarkPrice 并发送到 Pulsar
        match common::MarkPriceConverter::from_binance_futures(data, symbol) {
            Ok(mark_price) => {
                log::debug!(
                    "[Binance Futures {}] Mark Price 转换成功 - 标记价格: {}, 指数价格: {}, 资金费率: {:?}",
                    symbol, mark_price.mark_price, mark_price.index_price, mark_price.funding_rate
                );
                common::PulsarClient::publish_async(common::pulsar::mark_price::FUTURES_MARK_PRICE, mark_price);
            }
            Err(e) => log::error!("[Binance Futures {}] Mark Price 转换失败: {}", symbol, e),
        }
    }

    /// 处理深度数据
    fn handle_depth_data(stream: &str, data: &serde_json::Value) {
        let symbol = stream.split('@').next().unwrap_or("UNKNOWN").to_uppercase();
        let bids = data.get("b").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let asks = data.get("a").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        
        log::debug!("[{}] Depth - 买单: {}, 卖单: {}", symbol, bids, asks);
    }
}

impl Default for BinanceFutures {
    fn default() -> Self {
        Self::new()
    }
}
