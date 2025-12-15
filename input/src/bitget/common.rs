// Bitget WebSocket 通用模块
// 用于 Spot 和 Futures 的共享逻辑

use common::PulsarClient;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{StreamExt, SinkExt};
use serde_json::json;

/// WebSocket 重连配置
#[derive(Clone)]
pub struct ReconnectConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: u32,
    pub heartbeat_interval: Duration,
    pub heartbeat_timeout: Duration,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2,
            heartbeat_interval: Duration::from_secs(30),
            heartbeat_timeout: Duration::from_secs(60),
        }
    }
}

/// 计算指数退避延迟
pub fn calculate_backoff_delay(retry_count: u32, config: &ReconnectConfig) -> Duration {
    let delay_secs = config.initial_delay.as_secs() 
        * (config.backoff_factor.pow(retry_count.min(6)) as u64);
    Duration::from_secs(delay_secs.min(config.max_delay.as_secs()))
}

/// Bitget 产品类型
#[derive(Debug, Clone, Copy)]
pub enum BitgetInstType {
    /// 现货
    Spot,
    /// USDT永续合约
    UsdtFutures,
}

impl BitgetInstType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spot => "SPOT",
            Self::UsdtFutures => "USDT-FUTURES",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Spot => "现货",
            Self::UsdtFutures => "永续合约",
        }
    }
}

/// 交易对信息（通用）
pub trait BitgetSymbol {
    fn get_symbol(&self) -> Option<&String>;
}

/// Bitget WebSocket 通用客户端
pub struct BitgetWebSocket<T: BitgetSymbol> {
    inst_type: BitgetInstType,
    symbols: Arc<RwLock<Vec<T>>>,
    pulsar_client: Option<Arc<PulsarClient>>,
}

impl<T: BitgetSymbol + Clone + Send + Sync + 'static> BitgetWebSocket<T> {
    pub fn new(inst_type: BitgetInstType) -> Self {
        Self {
            inst_type,
            symbols: Arc::new(RwLock::new(Vec::new())),
            pulsar_client: None,
        }
    }

    /// 设置 PulsarClient
    pub fn with_pulsar(mut self, pulsar_client: Arc<PulsarClient>) -> Self {
        self.pulsar_client = Some(pulsar_client);
        self
    }

    pub async fn set_symbols(&self, symbols: Vec<T>) {
        let mut s = self.symbols.write().await;
        *s = symbols;
    }

    pub async fn get_symbols(&self) -> Vec<T> {
        self.symbols.read().await.clone()
    }

    pub async fn get_symbol_count(&self) -> usize {
        self.symbols.read().await.len()
    }

    /// 启动 WebSocket 连接
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let symbols = self.get_symbols().await;
        if symbols.is_empty() {
            log::warn!("没有加载到 Bitget {} 配置，跳过数据接入", self.inst_type.name());
            return Ok(());
        }

        log::info!("开始订阅 {} 个 Bitget {} 的实时数据", symbols.len(), self.inst_type.name());

        let inst_type = self.inst_type;
        let pulsar_client = self.pulsar_client.clone();
        tokio::spawn(async move {
            Self::run_websocket_loop(inst_type, symbols, pulsar_client).await;
        });

        log::info!("✅ Bitget {} 数据订阅任务已启动", self.inst_type.name());
        Ok(())
    }

    /// WebSocket 连接循环（内部处理重连，永不返回）
    async fn run_websocket_loop(inst_type: BitgetInstType, symbols: Vec<T>, pulsar_client: Option<Arc<PulsarClient>>) {
        let config = ReconnectConfig::default();
        let mut retry_count = 0;

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        'reconnect: loop {
            log::info!("🔌 正在连接到 Bitget {} WebSocket...", inst_type.name());

            let url = "wss://ws.bitget.com/v2/ws/public";
            let (ws_stream, _) = match connect_async(url).await {
                Ok(stream) => {
                    log::info!("✅ Bitget {} WebSocket 连接成功", inst_type.name());
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

            // 构建订阅参数
            let mut subscribe_args = Vec::new();
            for symbol_obj in &symbols {
                if let Some(symbol) = symbol_obj.get_symbol() {
                    let inst_id = symbol.replace("/", "");

                    // Ticker
                    subscribe_args.push(json!({
                        "instType": inst_type.as_str(),
                        "channel": "ticker",
                        "instId": inst_id
                    }));
                }
            }

            log::info!("[{}] 准备订阅 {} 个数据流", inst_type.name(), subscribe_args.len());

            // 分批订阅（Bitget 推荐每批50个）
            const BATCH_SIZE: usize = 50;
            let batches: Vec<_> = subscribe_args.chunks(BATCH_SIZE).collect();

            log::info!("[{}] 分 {} 批订阅，每批最多 {} 个频道", inst_type.name(), batches.len(), BATCH_SIZE);

            for (i, batch) in batches.iter().enumerate() {
                let subscribe_msg = json!({"op": "subscribe", "args": batch});
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    log::error!("[{}] 发送第 {} 批订阅失败: {}", inst_type.name(), i + 1, e);
                    retry_count += 1;
                    continue 'reconnect;
                }
                log::info!("[{}] ✅ 第 {}/{} 批订阅已发送（{} 个频道）", 
                    inst_type.name(), i + 1, batches.len(), batch.len());

                if i < batches.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            log::info!("[{}] ✅ 所有订阅请求已发送", inst_type.name());

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
                            log::warn!("[{}] 💔 心跳超时，主动断开重连", inst_type.name());
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        
                        // Bitget 使用文本 "ping"
                        if let Err(e) = write.send(Message::Text("ping".to_string())).await {
                            log::error!("[{}] 发送心跳失败: {}", inst_type.name(), e);
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        log::debug!("[{}] 💓 发送心跳 ping", inst_type.name());
                    }
                    
                    msg = read.next() => {
                        let msg = match msg {
                            Some(m) => m,
                            None => {
                                log::warn!("[{}] WebSocket 流已结束，准备重连", inst_type.name());
                                retry_count += 1;
                                continue 'reconnect;
                            }
                        };
                        
                        last_message_time = Instant::now();
                        
                        match msg {
                            Ok(Message::Text(text)) => {
                                message_count += 1;
                                
                                // Bitget 心跳响应
                                if text == "pong" {
                                    log::debug!("[{}] 💓 收到 pong", inst_type.name());
                                    continue;
                                }
                                
                                if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                    log::debug!("[{}] 收到消息 #{}: {}", inst_type.name(), message_count,
                                        if text.len() > 200 { &text[..200] } else { &text });
                                    last_log_time = Instant::now();
                                }
                                
                                // 解析 Bitget 消息
                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(event) = json_msg.get("event").and_then(|v| v.as_str()) {
                                        if event == "subscribe" {
                                            log::info!("[{}] 📩 订阅成功: {}", inst_type.name(), text);
                                            continue;
                                        } else if event == "error" {
                                            log::error!("[{}] ❌ 订阅错误: {}", inst_type.name(), text);
                                            continue;
                                        }
                                    }

                                    // 处理数据消息
                                    if let Some(arg) = json_msg.get("arg") {
                                        if let Some(data_array) = json_msg.get("data").and_then(|v| v.as_array()) {
                                            let channel = arg.get("channel").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let inst_id = arg.get("instId").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");

                                            for data in data_array {
                                                Self::handle_data(inst_type, channel, inst_id, data, pulsar_client.clone());
                                            }
                                        }
                                    }
                                }

                                if message_count % 1000 == 0 {
                                    log::debug!("[{}] 已接收 {} 条消息", inst_type.name(), message_count);
                                }
                            }
                            Ok(Message::Close(_)) => {
                                log::warn!("[{}] 收到 Close 帧，准备重连", inst_type.name());
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            Err(e) => {
                                log::error!("[{}] WebSocket 错误: {}, 准备重连", inst_type.name(), e);
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            _ => {}
                        }
                    }

                    _ = &mut ctrl_c => {
                        log::info!("[{}] 收到关闭信号，停止 WebSocket", inst_type.name());
                        break 'reconnect;
                    }
                }
            }
        }

        log::info!("[{}] Bitget WebSocket 守护任务已停止", inst_type.name());
    }

    /// 处理数据消息
    fn handle_data(inst_type: BitgetInstType, channel: &str, inst_id: &str, data: &serde_json::Value, _pulsar_client: Option<Arc<PulsarClient>>) {
        match channel {
            "ticker" => {
                log::debug!("[{} {}] Ticker - 原始数据: {:?}", inst_type.name(), inst_id, data);

                // 转换为 UnifiedTicker 并发送到 Pulsar
                let converter_result = match inst_type {
                    BitgetInstType::Spot => common::TickerConverter::from_bitget_spot(data, inst_id),
                    BitgetInstType::UsdtFutures => common::TickerConverter::from_bitget_futures(data, inst_id),
                };

                match converter_result {
                    Ok(unified_ticker) => {
                        log::debug!(
                            "[{} {}] 转换成功 - 价格: {}, 涨跌幅: {:?}%", 
                            inst_type.name(), inst_id, unified_ticker.close, unified_ticker.change_percent_24h
                        );
                        
                        let topic = match inst_type {
                            BitgetInstType::Spot => common::pulsar::ticker::SPOT_TICKER,
                            BitgetInstType::UsdtFutures => common::pulsar::ticker::FUTURES_TICKER,
                        };
                        common::PulsarClient::publish_async(topic, unified_ticker);
                    }
                    Err(e) => {
                        log::info!("接收到的数据是：{:?}", data);
                        log::error!("[{} {}] Ticker 转换失败: {}", inst_type.name(), inst_id, e);
                    }
                }
                
                // 对于 Futures，同时提取并发送 Mark Price 数据
                if matches!(inst_type, BitgetInstType::UsdtFutures) {
                    match common::MarkPriceConverter::from_bitget_futures(data, inst_id) {
                        Ok(mark_price) => {
                            log::debug!(
                                "[Bitget Futures {}] Mark Price 转换成功 - 标记价格: {}, 指数价格: {}, 资金费率: {:?}",
                                inst_id, mark_price.mark_price, mark_price.index_price, mark_price.funding_rate
                            );
                            common::PulsarClient::publish_async(common::pulsar::mark_price::FUTURES_MARK_PRICE, mark_price);
                        }
                        Err(e) => log::error!("[Bitget Futures {}] Mark Price 转换失败: {}", inst_id, e),
                    }
                }
            }
            ch if ch.starts_with("candle") => {
                if let Some(kline_data) = data.as_array() {
                    if kline_data.len() >= 5 {
                        let close = kline_data.get(4).and_then(|v| v.as_str()).unwrap_or("0");
                        log::debug!("[{} {}] Kline {} - 收盘价: {}",
                            inst_type.name(), inst_id, channel, close);
                    }
                }
            }
            ch if ch.starts_with("books") => {
                let asks = data.get("asks").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                let bids = data.get("bids").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                log::debug!("[{} {}] Depth - 买单: {}, 卖单: {}",
                    inst_type.name(), inst_id, bids, asks);
            }
            _ => {}
        }
    }
}
