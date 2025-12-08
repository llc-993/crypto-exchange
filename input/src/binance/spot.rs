// Binance 现货交易接入模块
use orm::entities::exchange::AppExchangeSpotCoin;
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
struct ReconnectConfig {
    /// 初始重连延迟
    initial_delay: Duration,
    /// 最大重连延迟
    max_delay: Duration,
    /// 退避因子
    backoff_factor: u32,
    /// 心跳间隔
    heartbeat_interval: Duration,
    /// 心跳超时
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

pub struct BinanceSpot {
    /// 交易对配置列表
    spot_coins: Arc<RwLock<Vec<AppExchangeSpotCoin>>>,
    pulsar_client: Option<Arc<PulsarClient>>,
}

impl BinanceSpot {
    pub fn new() -> Self {
        Self {
            spot_coins: Arc::new(RwLock::new(Vec::new())),
            pulsar_client: None,
        }
    }

    /// 设置 PulsarClient
    pub fn with_pulsar(mut self, pulsar_client: Arc<PulsarClient>) -> Self {
        self.pulsar_client = Some(pulsar_client);
        self
    }

    /// 从数据库加载 Binance 现货交易对配置
    pub async fn load_spot_coins(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("正在加载 Binance 现货交易对配置...");

        match AppExchangeSpotCoin::select_spot_coin_by_data_source("binance".to_string()).await {
            Ok(coin_list) => {
                let count = coin_list.len();

                let mut spot_coins = self.spot_coins.write().await;
                *spot_coins = coin_list;

                log::info!("✅ Binance 现货交易对加载完成，共 {} 个交易对", count);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Binance 现货交易对加载失败: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// 获取已加载的交易对数量
    pub async fn get_spot_coin_count(&self) -> usize {
        self.spot_coins.read().await.len()
    }

    /// 获取所有交易对配置（只读）
    pub async fn get_spot_coins(&self) -> Vec<AppExchangeSpotCoin> {
        self.spot_coins.read().await.clone()
    }

    /// 启动现货数据接入（带自动重连）
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Binance 现货数据接入服务启动中...");

        let spot_coins = self.get_spot_coins().await;
        if spot_coins.is_empty() {
            log::warn!("没有加载到 Binance 现货交易对，跳过数据接入");
            return Ok(());
        }

        log::info!("开始订阅 {} 个交易对的实时数据", spot_coins.len());

        // 启动带重连机制的 WebSocket 连接
        tokio::spawn(async move {
            Self::run_websocket_with_reconnect(spot_coins).await;
        });

        log::info!("✅ Binance 现货数据订阅任务已启动");
        Ok(())
    }

    /// 运行 WebSocket 连接（带自动重连和优雅关闭）
    async fn run_websocket_with_reconnect(spot_coins: Vec<AppExchangeSpotCoin>) {
        let config = ReconnectConfig::default();
        let mut retry_count = 0;

        // 监听 Ctrl+C 信号
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        'reconnect: loop {
            log::info!("🔌 正在连接到 Binance WebSocket...");

            // 创建 WebSocket 任务
            let ws_future = Self::run_websocket(spot_coins.clone(), &config);
            tokio::pin!(ws_future);

            // 使用 select! 同时监听 WebSocket 和关闭信号
            tokio::select! {
                result = &mut ws_future => {
                    match result {
                        Ok(_) => {
                            log::info!("WebSocket 连接正常关闭");
                            retry_count = 0; // 重置重试计数
                        }
                        Err(e) => {
                            log::error!("❌ WebSocket 连接错误: {}", e);
                        }
                    }
                }
                _ = &mut ctrl_c => {
                    log::info!("收到关闭信号，停止 WebSocket 重连");
                    break 'reconnect;
                }
            }

            // 计算退避延迟
            let delay = calculate_backoff_delay(retry_count, &config);
            log::warn!("⏳ {}秒后重新连接 (第{}次重试)...", delay.as_secs(), retry_count + 1);

            // 等待重连延迟，同时监听关闭信号
            tokio::select! {
                _ = tokio::time::sleep(delay) => {
                    retry_count += 1;
                }
                _ = &mut ctrl_c => {
                    log::info!("收到关闭信号，取消重连");
                    break 'reconnect;
                }
            }
        }

        log::info!("Binance WebSocket 守护任务已停止");
    }


    /// 运行 WebSocket 连接（单连接多订阅，带心跳监控）
    async fn run_websocket(
        spot_coins: Vec<AppExchangeSpotCoin>,
        config: &ReconnectConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {


        // 连接到 Binance WebSocket
        let url = "wss://stream.binance.com:9443/ws";
        log::info!("正在连接到 Binance WebSocket: {}", url);

        let (ws_stream, _) = connect_async(url).await?;
        log::info!("✅ Binance WebSocket 连接成功");

        let (mut write, mut read) = ws_stream.split();

        // 构建订阅参数
        let mut subscribe_params = Vec::new();

        log::info!("准备为 {} 个交易对构建订阅", spot_coins.len());

        for coin in &spot_coins {
            if let Some(symbol) = &coin.symbol {
                // 移除斜杠，Binance 使用 BTCUSDT 格式而不是 BTC/USDT
                let symbol_lower = symbol.replace("/", "").to_lowercase();

                // 订阅 24小时 Ticker
                subscribe_params.push(format!("{}@ticker", symbol_lower));

                // 订阅深度数据（10档）
                /* subscribe_params.push(format!("{}@depth10@100ms", symbol_lower));

                 // 订阅 K线数据
                 let intervals = KlineInterval::all();

                 for interval in intervals {
                     let binance_interval = interval.binance_interval();
                     subscribe_params.push(format!("{}@kline_{}", symbol_lower, binance_interval));
                 }*/
            }
        }

        log::info!("准备订阅 {} 个数据流", subscribe_params.len());

        // 发送订阅请求
        let subscribe_msg = json!({
            "method": "SUBSCRIBE",
            "params": subscribe_params,
            "id": 1
        });

        write.send(Message::Text(subscribe_msg.to_string())).await?;
        log::info!("✅ 订阅请求已发送");

        // 处理接收到的消息（带心跳监控）
        let mut message_count = 0;
        let mut last_log_time = Instant::now();
        let mut last_message_time = Instant::now();
        let mut heartbeat_timer = interval(config.heartbeat_interval);
        heartbeat_timer.tick().await; // 跳过第一次立即触发

        loop {
            tokio::select! {
                // 心跳定时器
                _ = heartbeat_timer.tick() => {
                    // 检查是否超时
                    if last_message_time.elapsed() > config.heartbeat_timeout {
                        log::warn!("💔 心跳超时 ({}秒无消息)，主动断开连接", last_message_time.elapsed().as_secs());
                        break;
                    }
                    
                    // 发送 ping
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        log::error!("发送心跳失败: {}", e);
                        break;
                    }
                    log::debug!("💓 发送心跳 ping");
                }
                
                // 接收消息
                msg = read.next() => {
                    let msg = match msg {
                        Some(m) => m,
                        None => {
                            log::warn!("WebSocket 流已关闭");
                            break;
                        }
                    };
                    
                    // 更新最后接收消息时间
                    last_message_time = Instant::now();
                    
                    match msg {
                        Ok(Message::Text(text)) => {
                            message_count += 1;
                            
                            // 每收到一条消息就记录（前10条）或每10秒记录一次
                            if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                log::debug!("收到消息 #{}: {}", message_count, if text.len() > 200 { &text[..200] } else { &text });
                                last_log_time = Instant::now();
                            }
                            
                            // 解析 JSON 消息
                            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                // 检查是否是订阅响应
                                if json_msg.get("result").is_some() {
                                    log::info!("📩 订阅响应: {}", text);
                                    continue;
                                }
                                
                                // 检查是否有错误
                                if let Some(error) = json_msg.get("error") {
                                    log::error!("❌ 订阅错误: {}", error);
                                    continue;
                                }
                                
                                // 处理数据流消息
                                if let Some(event_type) = json_msg.get("e").and_then(|v| v.as_str()) {
                                    let symbol = json_msg.get("s").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
                                    
                                    match event_type {
                                        "24hrTicker" => {
                                            let price = json_msg.get("c").and_then(|v| v.as_str()).unwrap_or("0");
                                            let volume = json_msg.get("v").and_then(|v| v.as_str()).unwrap_or("0");
                                            log::debug!("[{}] Ticker - 价格: {}, 24h成交量: {}", symbol, price, volume);
                                            match common::TickerConverter::from_binance_spot(&json_msg, symbol) {
                                                Ok(unified_ticker) => {
                                                    common::PulsarClient::publish_async("spot-ticker", unified_ticker);
                                                }
                                                Err(e) => log::error!("[Binance Spot {}] Ticker 转换失败: {}", symbol, e),
                                            }
                                        }
                                        "depthUpdate" => {
                                            let bids = json_msg.get("b").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                            let asks = json_msg.get("a").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                            log::debug!("[{}] Depth - 买单: {}, 卖单: {}", symbol, bids, asks);
                                        }
                                        "kline" => {
                                            if let Some(k) = json_msg.get("k") {
                                                let interval = k.get("i").and_then(|v| v.as_str()).unwrap_or("unknown");
                                                let close = k.get("c").and_then(|v| v.as_str()).unwrap_or("0");
                                                let is_closed = k.get("x").and_then(|v| v.as_bool()).unwrap_or(false);
                                                log::debug!("[{}] Kline {} - 收盘价: {}, 已完成: {}", symbol, interval, close, is_closed);
                                            }
                                        }
                                        _ => {
                                            log::debug!("未知事件类型: {}", event_type);
                                        }
                                    }
                                }
                            }
                            
                            // 每1000条消息打印一次统计
                            if message_count % 1000 == 0 {
                                log::debug!("已接收 {} 条消息", message_count);
                            }
                        }
                        Ok(Message::Ping(payload)) => {
                            write.send(Message::Pong(payload)).await?;
                            log::debug!("收到 Ping，已回复 Pong");
                        }
                        Ok(Message::Pong(_)) => {
                            log::debug!("💓 收到 Pong");
                        }
                        Ok(Message::Close(_)) => {
                            log::warn!("WebSocket 连接已关闭");
                            break;
                        }
                        Err(e) => {
                            log::error!("WebSocket 错误: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        log::info!("WebSocket 连接已断开，总共接收 {} 条消息", message_count);
        Ok(())
    }
}

impl Default for BinanceSpot {
    fn default() -> Self {
        Self::new()
    }
}
