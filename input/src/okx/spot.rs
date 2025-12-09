// OKX 现货交易接入模块
use orm::entities::exchange::AppExchangeSpotCoin;
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

pub struct OkxSpot {
    spot_coins: Arc<RwLock<Vec<AppExchangeSpotCoin>>>,
    pulsar_client: Option<Arc<PulsarClient>>,
}

impl OkxSpot {
    pub fn new() -> Self {
        Self {
            spot_coins: Arc::new(RwLock::new(Vec::new())),
            pulsar_client: None,
        }
    }

    /// 设置 PulsarClient
    pub fn with_pulsar(mut self, _pulsar_client: Option<Arc<PulsarClient>>) -> Self {
        self.pulsar_client = _pulsar_client;
        self
    }

    pub async fn load_spot_coins(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("正在加载 OKX 现货交易对配置...");
        
        match AppExchangeSpotCoin::select_spot_coin_by_data_source("binance".to_string()).await {
            Ok(coin_list) => {
                let count = coin_list.len();
                let mut spot_coins = self.spot_coins.write().await;
                *spot_coins = coin_list;
                log::info!("✅ OKX 现货交易对加载完成，共 {} 个交易对", count);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ OKX 现货交易对加载失败: {}", e);
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
        log::info!("OKX 现货数据接入服务启动中...");
        
        let spot_coins = self.get_spot_coins().await;
        if spot_coins.is_empty() {
            log::warn!("没有加载到 OKX 现货交易对，跳过数据接入");
            return Ok(());
        }
        
        log::info!("开始订阅 {} 个交易对的实时数据", spot_coins.len());
        
        // OKX 需要两个连接：
        // 1. /ws/v5/public - ticker 和 depth
        // 2. /ws/v5/business - K线数据
        let coins_for_public = spot_coins.clone();
        let _coins_for_business = spot_coins.clone();
        let pulsar_client = self.pulsar_client.clone();
        
        tokio::spawn(async move {
            Self::run_public_websocket(coins_for_public, pulsar_client).await;
        });
        
      /*  tokio::spawn(async move {
            Self::run_business_websocket(coins_for_business).await;
        });*/
        
        log::info!("✅ OKX 现货数据订阅任务已启动（public + business 双连接）");
        Ok(())
    }
    
    /// 运行 Public WebSocket（ticker + depth）
    async fn run_public_websocket(spot_coins: Vec<AppExchangeSpotCoin>, _pulsar_client: Option<Arc<PulsarClient>>) {
        
        let config = ReconnectConfig::default();
        let mut retry_count = 0;
        let mut request_id = 1u64;
        
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);
        
        'reconnect: loop {
            log::info!("🔌 正在连接到 OKX Public WebSocket...");
            
            let url = "wss://ws.okx.com:8443/ws/v5/public";
            let (ws_stream, _) = match connect_async(url).await {
                Ok(stream) => {
                    log::info!("✅ OKX Public WebSocket 连接成功");
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
            
            // 构建订阅（ticker + depth，不包括 K线）
            let mut subscribe_args = Vec::new();
            for coin in &spot_coins {
                if let Some(symbol) = &coin.symbol {
                    // OKX 使用 BTC-USDT 格式（带横杠）
                    let inst_id = symbol.replace("/", "-");
                    
                    // Ticker
                    subscribe_args.push(json!({
                        "channel": "tickers",
                        "instId": inst_id
                    }));
                    
                    // Depth (5档，OKX 使用 books5)
                   /* subscribe_args.push(json!({
                        "channel": "books5",
                        "instId": inst_id
                    }));*/
                }
            }
            
            log::info!("[Public] 准备订阅 {} 个数据流", subscribe_args.len());
            
            // 分批订阅，每批 50 个
            const BATCH_SIZE: usize = 50;
            let batches: Vec<_> = subscribe_args.chunks(BATCH_SIZE).collect();
            
            log::debug!("[Public] 分 {} 批订阅，每批最多 {} 个频道", batches.len(), BATCH_SIZE);
            
            for (i, batch) in batches.iter().enumerate() {
                let subscribe_msg = json!({
                    "id": request_id.to_string(),
                    "op": "subscribe",
                    "args": batch
                });
                request_id += 1;
                
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    log::error!("[Public] 发送第 {} 批订阅失败: {}", i + 1, e);
                    retry_count += 1;
                    continue 'reconnect;
                }
                log::info!("[Public] ✅ 第 {}/{} 批订阅已发送（{} 个频道）", i + 1, batches.len(), batch.len());
                
                if i < batches.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            
            log::info!("[Public] ✅ 所有订阅请求已发送");
            
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
                            log::warn!("[Public] 💔 心跳超时，主动断开重连");
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        
                        // OKX 使用标准 WebSocket Ping
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            log::error!("[Public] 发送心跳失败: {}", e);
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        log::debug!("[Public] 💓 发送心跳 ping");
                    }
                    
                    msg = read.next() => {
                        let msg = match msg {
                            Some(m) => m,
                            None => {
                                log::warn!("[Public] WebSocket 流已结束，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                        };
                        
                        last_message_time = Instant::now();
                        
                        match msg {
                            Ok(Message::Text(text)) => {
                                message_count += 1;
                                
                                if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                    log::debug!("[Public] 收到消息 #{}: {}", message_count, if text.len() > 200 { &text[..200] } else { &text });
                                    last_log_time = Instant::now();
                                }
                                
                                // 解析 OKX 消息
                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(event) = json_msg.get("event").and_then(|v| v.as_str()) {
                                        if event == "subscribe" {
                                            log::info!("[Public] 📩 订阅成功: {}", text);
                                            continue;
                                        } else if event == "error" {
                                            log::error!("[Public] ❌ 订阅错误: {}", text);
                                            continue;
                                        }
                                    }
                                    
                                    if let Some(arg) = json_msg.get("arg") {
                                        if let Some(data_array) = json_msg.get("data").and_then(|v| v.as_array()) {
                                            let channel = arg.get("channel").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let inst_id = arg.get("instId").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
                                            
                                            for data in data_array {
                                                match channel {
                                                    "tickers" => {
                                                        log::debug!("[OKX Spot {}] Ticker - 原始数据: {:?}", inst_id, data);

                                                        // 转换为 UnifiedTicker 并发送到 Pulsar
                                                        match common::TickerConverter::from_okx_spot(data, inst_id) {
                                                            Ok(unified_ticker) => {
                                                                log::debug!(
                                                                    "[OKX Spot {}] 转换成功 - 价格: {}, 涨跌幅: {:?}%", 
                                                                    inst_id, unified_ticker.close, unified_ticker.change_percent_24h
                                                                );
                                                                PulsarClient::publish_async(topics::ticker::SPOT_TICKER, unified_ticker);
                                                            }
                                                            Err(e) => log::error!("[OKX Spot {}] Ticker 转换失败: {}", inst_id, e),
                                                        }
                                                    }
                                                    ch if ch.starts_with("books") => {
                                                        let asks = data.get("asks").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                                        let bids = data.get("bids").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                                        log::debug!("[{}] Depth - 买单: {}, 卖单: {}", inst_id, bids, asks);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if message_count % 1000 == 0 {
                                    log::debug!("[Public] 已接收 {} 条消息", message_count);
                                }
                            }
                            Ok(Message::Ping(payload)) => {
                                if let Err(e) = write.send(Message::Pong(payload)).await {
                                    log::error!("[Public] 回复 Pong 失败: {}", e);
                                    retry_count += 1;
                                    continue 'reconnect;
                                }
                                log::debug!("[Public] 收到 Ping，已回复 Pong");
                            }
                            Ok(Message::Pong(_)) => {
                                log::debug!("[Public] 💓 收到 Pong");
                            }
                            Ok(Message::Close(_)) => {
                                log::warn!("[Public] 收到 Close 帧，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            Err(e) => {
                                log::error!("[Public] WebSocket 错误: {}, 准备重连", e);
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            _ => {}
                        }
                    }
                    
                    _ = &mut ctrl_c => {
                        log::info!("[Public] 收到关闭信号，停止 WebSocket");
                        break 'reconnect;
                    }
                }
            }
        }
        
        log::info!("[Public] OKX Public WebSocket 守护任务已停止");
    }
    
    /// 运行 Business WebSocket（K线数据）
    async fn run_business_websocket(spot_coins: Vec<AppExchangeSpotCoin>) {
        use common::KlineInterval;
        
        let config = ReconnectConfig::default();
        let mut retry_count = 0;
        let mut request_id = 10000u64; // 使用不同的 ID 范围
        
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);
        
        'reconnect: loop {
            log::info!("🔌 正在连接到 OKX Business WebSocket...");
            
            let url = "wss://ws.okx.com:8443/ws/v5/business";
            let (ws_stream, _) = match connect_async(url).await {
                Ok(stream) => {
                    log::info!("✅ OKX Business WebSocket 连接成功");
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
            
            // 构建 K线订阅
            let mut subscribe_args = Vec::new();
            for coin in &spot_coins {
                if let Some(symbol) = &coin.symbol {
                    let inst_id = symbol.replace("/", "-");
                    
                    let intervals = KlineInterval::all();
                    for interval in intervals {
                        let channel = interval.okx_interval();
                        subscribe_args.push(json!({
                            "channel": channel,
                            "instId": inst_id
                        }));
                    }
                }
            }
            
            log::info!("[Business] 准备订阅 {} 个 K线数据流", subscribe_args.len());
            
            const BATCH_SIZE: usize = 50;
            let batches: Vec<_> = subscribe_args.chunks(BATCH_SIZE).collect();
            
            log::info!("[Business] 分 {} 批订阅，每批最多 {} 个频道", batches.len(), BATCH_SIZE);
            
            for (i, batch) in batches.iter().enumerate() {
                let subscribe_msg = json!({
                    "id": request_id.to_string(),
                    "op": "subscribe",
                    "args": batch
                });
                request_id += 1;
                
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    log::error!("[Business] 发送第 {} 批订阅失败: {}", i + 1, e);
                    retry_count += 1;
                    continue 'reconnect;
                }
                log::info!("[Business] ✅ 第 {}/{} 批订阅已发送（{} 个频道）", i + 1, batches.len(), batch.len());
                
                if i < batches.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            
            log::info!("[Business] ✅ 所有 K线订阅请求已发送");
            
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
                            log::warn!("[Business] 💔 心跳超时，主动断开重连");
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            log::error!("[Business] 发送心跳失败: {}", e);
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        log::debug!("[Business] 💓 发送心跳 ping");
                    }
                    
                    msg = read.next() => {
                        let msg = match msg {
                            Some(m) => m,
                            None => {
                                log::warn!("[Business] WebSocket 流已结束，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                        };
                        
                        last_message_time = Instant::now();
                        
                        match msg {
                            Ok(Message::Text(text)) => {
                                message_count += 1;
                                
                                if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                    log::info!("[Business] 收到消息 #{}: {}", message_count, if text.len() > 200 { &text[..200] } else { &text });
                                    last_log_time = Instant::now();
                                }
                                
                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(event) = json_msg.get("event").and_then(|v| v.as_str()) {
                                        if event == "subscribe" {
                                            log::info!("[Business] 📩 订阅成功: {}", text);
                                            continue;
                                        } else if event == "error" {
                                            log::error!("[Business] ❌ 订阅错误: {}", text);
                                            continue;
                                        }
                                    }
                                    
                                    if let Some(arg) = json_msg.get("arg") {
                                        if let Some(data_array) = json_msg.get("data").and_then(|v| v.as_array()) {
                                            let channel = arg.get("channel").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            let inst_id = arg.get("instId").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
                                            
                                            for data in data_array {
                                                if channel.starts_with("candle") {
                                                    if let Some(kline_data) = data.as_array() {
                                                        if kline_data.len() >= 5 {
                                                            let close = kline_data.get(4).and_then(|v| v.as_str()).unwrap_or("0");
                                                            log::info!("[{}] Kline {} - 收盘价: {}", inst_id, channel, close);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if message_count % 1000 == 0 {
                                    log::info!("[Business] 已接收 {} 条消息", message_count);
                                }
                            }
                            Ok(Message::Ping(payload)) => {
                                if let Err(e) = write.send(Message::Pong(payload)).await {
                                    log::error!("[Business] 回复 Pong 失败: {}", e);
                                    retry_count += 1;
                                    continue 'reconnect;
                                }
                                log::debug!("[Business] 收到 Ping，已回复 Pong");
                            }
                            Ok(Message::Pong(_)) => {
                                log::debug!("[Business] 💓 收到 Pong");
                            }
                            Ok(Message::Close(_)) => {
                                log::warn!("[Business] 收到 Close 帧，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            Err(e) => {
                                log::error!("[Business] WebSocket 错误: {}, 准备重连", e);
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            _ => {}
                        }
                    }
                    
                    _ = &mut ctrl_c => {
                        log::info!("[Business] 收到关闭信号，停止 WebSocket");
                        break 'reconnect;
                    }
                }
            }
        }
        
        log::info!("[Business] OKX Business WebSocket 守护任务已停止");
    }
}

impl Default for OkxSpot {
    fn default() -> Self {
        Self::new()
    }
}
