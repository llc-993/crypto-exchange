// Bitget 现货交易接入模块
use orm::entities::exchange::AppExchangeSpotCoin;
use common::PulsarClient;
use super::common::BitgetSymbol;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{StreamExt, SinkExt};
use serde_json::json;

// 为 AppExchangeSpotCoin 实现 BitgetSymbol trait
impl BitgetSymbol for AppExchangeSpotCoin {
    fn get_symbol(&self) -> Option<&String> {
        self.symbol.as_ref()
    }
}

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
            heartbeat_interval: Duration::from_secs(30),
            heartbeat_timeout: Duration::from_secs(60),
        }
    }
}

/// 计算指数退避延迟
fn calculate_backoff_delay(retry_count: u32, config: &ReconnectConfig) -> Duration {
    let delay_secs = config.initial_delay.as_secs() 
        * (config.backoff_factor.pow(retry_count.min(6)) as u64);
    Duration::from_secs(delay_secs.min(config.max_delay.as_secs()))
}

pub struct BitgetSpot {
    spot_coins: Arc<RwLock<Vec<AppExchangeSpotCoin>>>,
}

impl BitgetSpot {
    pub fn new() -> Self {
        Self {
            spot_coins: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 设置 PulsarClient
    pub async fn run_pulsar(self, _pulsar_client: Arc<PulsarClient>) -> Self {
        // Bitget Spot 使用直接存储，不使用 BitgetWebSocket
        // 需要添加 pulsar_client 字段
        self
    }

    pub async fn load_spot_coins(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("正在加载 Bitget 现货交易对配置...");

        match AppExchangeSpotCoin::select_spot_coin_by_data_source("binance".to_string()).await {
            Ok(coin_list) => {
                let count = coin_list.len();
                let mut spot_coins = self.spot_coins.write().await;
                *spot_coins = coin_list;
                log::info!("✅ Bitget 现货交易对加载完成，共 {} 个交易对", count);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Bitget 现货交易对加载失败: {}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn get_spot_coin_count(&self) -> usize {
        self.spot_coins.read().await.len()
    }

    pub async fn get_spot_coins(&self) -> Vec<AppExchangeSpotCoin> {
        self.spot_coins.read().await.clone()
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Bitget 现货数据接入服务启动中...");

        let spot_coins = self.get_spot_coins().await;
        if spot_coins.is_empty() {
            log::warn!("没有加载到 Bitget 现货交易对，跳过数据接入");
            return Ok(());
        }

        log::info!("开始订阅 {} 个交易对的实时数据", spot_coins.len());

        tokio::spawn(async move {
            Self::run_websocket_loop(spot_coins).await;
        });

        log::info!("✅ Bitget 现货数据订阅任务已启动");
        Ok(())
    }

    /// WebSocket 连接循环（内部处理重连，永不返回）
    async fn run_websocket_loop(spot_coins: Vec<AppExchangeSpotCoin>) {


        let config = ReconnectConfig::default();
        let mut retry_count = 0;

        // 监听 Ctrl+C 信号
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        'reconnect: loop {
            log::info!("🔌 正在连接到 Bitget WebSocket...");

            let url = "wss://ws.bitget.com/v2/ws/public";
            let (ws_stream, _) = match connect_async(url).await {
                Ok(stream) => {
                    log::info!("✅ Bitget WebSocket 连接成功");
                    retry_count = 0; // 连接成功，重置重试计数
                    stream
                }
                Err(e) => {
                    log::error!("❌ 连接失败: {}", e);
                    let delay = calculate_backoff_delay(retry_count, &config);
                    log::warn!("⏳ {}秒后重新连接 (第{}次重试)...", delay.as_secs(), retry_count + 1);

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

            // 构建订阅
            let mut subscribe_args = Vec::new();
            for coin in &spot_coins {
                if let Some(symbol) = &coin.symbol {
                    let inst_id = symbol.replace("/", "");

                    subscribe_args.push(json!({
                        "instType": "SPOT",
                        "channel": "ticker",
                        "instId": inst_id
                    }));

                   /* subscribe_args.push(json!({
                        "instType": "SPOT",
                        "channel": "books15",
                        "instId": inst_id
                    }));

                    let intervals = KlineInterval::all();
                    for interval in intervals {
                        let channel = interval.bitget_interval();
                        subscribe_args.push(json!({
                            "instType": "SPOT",
                            "channel": channel,
                            "instId": inst_id
                        }));
                    }*/
                }
            }

            log::info!("准备订阅 {} 个数据流", subscribe_args.len());

            // Bitget 限制：
            // - 单个连接最多 1000 个频道
            // - 强烈建议不超过 50 个频道（稳定性）
            // - 每秒最多 10 个消息
            // 因此分批订阅，每批 50 个，间隔 100ms
            const BATCH_SIZE: usize = 50;
            let batches: Vec<_> = subscribe_args.chunks(BATCH_SIZE).collect();

            log::info!("分 {} 批订阅，每批最多 {} 个频道", batches.len(), BATCH_SIZE);

            for (i, batch) in batches.iter().enumerate() {
                let subscribe_msg = json!({"op": "subscribe", "args": batch});
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    log::error!("发送第 {} 批订阅失败: {}", i + 1, e);
                    retry_count += 1;
                    continue 'reconnect;
                }
                log::info!("✅ 第 {}/{} 批订阅已发送（{} 个频道）", i + 1, batches.len(), batch.len());

                // 批次间延迟 100ms，避免超过每秒 10 个消息限制
                if i < batches.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            log::info!("✅ 所有订阅请求已发送");

            // 消息处理循环
            let mut message_count = 0;
            let mut last_log_time = Instant::now();
            let mut last_message_time = Instant::now();
            let mut heartbeat_timer = interval(config.heartbeat_interval);
            heartbeat_timer.tick().await;

            log::info!("🔄 开始消息处理循环，心跳间隔: {}s，超时: {}s",
                config.heartbeat_interval.as_secs(),
                config.heartbeat_timeout.as_secs());

            let _loop_start = Instant::now();

            loop {
                tokio::select! {
                    _ = heartbeat_timer.tick() => {
                        if last_message_time.elapsed() > config.heartbeat_timeout {
                            log::warn!("💔 心跳超时，主动断开重连");
                            retry_count += 1;
                            continue 'reconnect; // 直接跳到重连
                        }

                        if let Err(e) = write.send(Message::Text("ping".to_string())).await {
                            log::error!("发送心跳失败: {}", e);
                            retry_count += 1;
                            continue 'reconnect; // 直接跳到重连
                        }
                        log::debug!("💓 发送心跳 ping");
                    }

                    msg = read.next() => {
                        let msg = match msg {
                            Some(m) => m,
                            None => {
                                log::warn!("WebSocket 流已结束，准备重连");
                                retry_count += 1;
                                continue 'reconnect; // 跳到外层重连循环
                            }
                        };

                        last_message_time = Instant::now();

                        match msg {
                            Ok(Message::Text(text)) => {
                                message_count += 1;

                                if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                    log::debug!("收到消息 #{}: {}", message_count, if text.len() > 200 { &text[..200] } else { &text });
                                    last_log_time = Instant::now();
                                }

                                if text == "pong" {
                                    log::debug!("💓 收到 pong");
                                    continue;
                                }

                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(event) = json_msg.get("event").and_then(|v| v.as_str()) {
                                        if event == "subscribe" {
                                            log::info!("📩 订阅成功: {}", text);
                                            continue;
                                        } else if event == "error" {
                                            log::error!("❌ 订阅错误: {}", text);
                                            continue;
                                        }
                                    }

                                    if let Some(arg) = json_msg.get("arg") {
                                        if let Some(data_array) = json_msg.get("data").and_then(|v| v.as_array()) {
                                            let channel = arg.get("channel").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let inst_id = arg.get("instId").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");

                                            for data in data_array {
                                                match channel {
                                                    "ticker" => {
                                                        let last = data.get("last").and_then(|v| v.as_str()).unwrap_or("0");
                                                        let base_vol = data.get("baseVol").and_then(|v| v.as_str()).unwrap_or("0");
                                                        log::debug!("[{}] Ticker - 价格: {}, 24h成交量: {}", inst_id, last, base_vol);
                                                    }
                                                    ch if ch.starts_with("books") => {
                                                        let asks = data.get("asks").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                                        let bids = data.get("bids").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                                        log::debug!("[{}] Depth - 买单: {}, 卖单: {}", inst_id, bids, asks);
                                                    }
                                                    ch if ch.starts_with("candle") => {
                                                        if let Some(kline_data) = data.as_array() {
                                                            if kline_data.len() >= 5 {
                                                                let close = kline_data.get(4).and_then(|v| v.as_str()).unwrap_or("0");
                                                                log::debug!("[{}] Kline {} - 收盘价: {}", inst_id, ch, close);
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
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
                                continue 'reconnect; // 直接跳到重连，不返回
                            }
                            Err(e) => {
                                log::error!("WebSocket 错误: {}, 准备重连", e);
                                retry_count += 1;
                                continue 'reconnect; // 遇到错误时重连
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

            // 内层循环退出，记录原因
            // let connection_duration = _loop_start.elapsed();
          //  log::warn!("⚠️ 消息处理循环退出，连接持续时间: {:.2}秒，接收消息数: {}",
           //     connection_duration.as_secs_f64(), message_count);
        }
        
        log::info!("Bitget WebSocket 守护任务已停止");
    }
}

impl Default for BitgetSpot {
    fn default() -> Self {
        Self::new()
    }
}
