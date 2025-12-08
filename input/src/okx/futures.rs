// OKX 永续合约交易接入模块
use orm::entities::exchange::AppExchangeFuturesCoin;
use common::PulsarClient;
use std::sync::Arc;

/// OKX 永续合约接入
pub struct OkxFutures {
    futures_coins: Arc<tokio::sync::RwLock<Vec<AppExchangeFuturesCoin>>>,
    pulsar_client: Option<Arc<PulsarClient>>,
}

impl OkxFutures {
    pub fn new() -> Self {
        Self {
            futures_coins: Arc::new(tokio::sync::RwLock::new(Vec::new())),
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
        log::info!("正在加载 OKX 永续合约配置...");

        match AppExchangeFuturesCoin::select_futures_coin_by_exchange("okx".to_string()).await {
            Ok(coin_list) => {
                let count = coin_list.len();
                let mut futures_coins = self.futures_coins.write().await;
                *futures_coins = coin_list;
                log::info!("✅ OKX 永续合约配置加载完成，共 {} 个合约", count);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ OKX 永续合约配置加载失败: {}", e);
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
        log::info!("OKX 永续合约数据接入服务启动中...");

        let futures_coins = self.get_futures_coins().await;
        if futures_coins.is_empty() {
            log::warn!("没有加载到 OKX 永续合约配置，跳过数据接入");
            return Ok(());
        }

        log::info!("开始订阅 {} 个永续合约的实时数据", futures_coins.len());

        // OKX Futures 使用单个 public 连接
        // 包含: ticker, depth, kline, mark-price, funding-rate
        let pulsar_client = self.pulsar_client.clone();
        tokio::spawn(async move {
            Self::run_websocket_loop(futures_coins, pulsar_client).await;
        });

        log::info!("✅ OKX 永续合约数据订阅任务已启动");
        Ok(())
    }

    /// WebSocket 连接循环
    async fn run_websocket_loop(futures_coins: Vec<AppExchangeFuturesCoin>, pulsar_client: Option<Arc<PulsarClient>>) {
        
        use std::time::{Duration, Instant};
        use tokio::time::interval;
        use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
        use futures::{StreamExt, SinkExt};
        use serde_json::json;

        let mut retry_count = 0;
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        'reconnect: loop {
            log::info!("🔌 正在连接到 OKX Futures WebSocket...");

            let url = "wss://ws.okx.com:8443/ws/v5/public";
            let (ws_stream, _) = match connect_async(url).await {
                Ok(stream) => {
                    log::info!("✅ OKX Futures WebSocket 连接成功");
                    retry_count = 0;
                    stream
                }
                Err(e) => {
                    log::error!("❌ 连接失败: {}", e);
                    let delay = Duration::from_secs(2u64.pow(retry_count.min(6)));
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
            for coin in &futures_coins {
                // OKX 永续合约格式：BTC-USDT（不需要 -SWAP 后缀）
                // 数据库可能是: BTCUSDT 或 BTC/USDT
                // OKX 需要: BTC-USDT
                let symbol = &coin.symbol;
                let inst_id = if symbol.contains('/') {
                    // BTC/USDT -> BTC-USDT
                    symbol.replace("/", "-")
                } else {
                    // BTCUSDT -> BTC-USDT
                    // 假设格式为 XXXUSDT，需要分割
                    if let Some(pos) = symbol.rfind("USDT") {
                        let base = &symbol[..pos];
                        format!("{}-USDT", base)
                    } else if let Some(pos) = symbol.rfind("USD") {
                        let base = &symbol[..pos];
                        format!("{}-USD", base)
                    } else {
                        log::warn!("[OKX Futures] 无法解析 symbol: {}", symbol);
                        continue;
                    }
                };

                log::debug!("[OKX Futures] Symbol: {} -> instId: {}", symbol, inst_id);

                // 1. Ticker
                subscribe_args.push(json!({"channel": "tickers", "instId": inst_id}));

                // 2. Mark Price (标记价格)
                subscribe_args.push(json!({"channel": "mark-price", "instId": inst_id}));

                // 3. Funding Rate (资金费率)
                subscribe_args.push(json!({"channel": "funding-rate", "instId": inst_id}));

                // 4. K线数据 (暂时注释)
                // subscribe_args.push(json!({"channel": "funding-rate", "instId": inst_id}));

                // 4. K线 - OKX 使用小写格式：candle1m, candle1H, candle1D
                /* for interval in KlineInterval::all() {
                     let channel = match interval.binance_interval() {
                         "1m" => "candle1m",
                         "5m" => "candle5m",
                         "15m" => "candle15m",
                         "30m" => "candle30m",
                         "1h" => "candle1H",
                         "1d" => "candle1D",
                         "1w" => "candle1W",
                         "1M" => "candle1M",
                         _ => continue,
                     };
                     subscribe_args.push(json!({"channel": channel, "instId": inst_id}));
                 }*/

                // 5. Depth (5档)
                //  subscribe_args.push(json!({"channel": "books5", "instId": inst_id}));
            }

            log::info!("[OKX Futures] 准备订阅 {} 个数据流", subscribe_args.len());

            // 分批订阅
            const BATCH_SIZE: usize = 50;
            let batches: Vec<_> = subscribe_args.chunks(BATCH_SIZE).collect();

            for (i, batch) in batches.iter().enumerate() {
                let subscribe_msg = json!({"op": "subscribe", "args": batch});
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    log::error!("[OKX Futures] 发送第 {} 批订阅失败: {}", i + 1, e);
                    retry_count += 1;
                    continue 'reconnect;
                }
                log::info!("[OKX Futures] ✅ 第 {}/{} 批订阅已发送（{} 个频道）", 
                    i + 1, batches.len(), batch.len());

                if i < batches.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            // 消息处理循环
            let mut message_count = 0;
            let mut last_log_time = Instant::now();
            let mut last_message_time = Instant::now();
            let mut heartbeat_timer = interval(Duration::from_secs(30));
            heartbeat_timer.tick().await;

            loop {
                tokio::select! {
                    _ = heartbeat_timer.tick() => {
                        if last_message_time.elapsed() > Duration::from_secs(60) {
                            log::warn!("[OKX Futures] 💔 心跳超时，主动断开重连");
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            log::error!("[OKX Futures] 发送心跳失败: {}", e);
                            retry_count += 1;
                            continue 'reconnect;
                        }
                        log::debug!("[OKX Futures] 💓 发送心跳 ping");
                    }
                    
                    msg = read.next() => {
                        let msg = match msg {
                            Some(m) => m,
                            None => {
                                log::warn!("[OKX Futures] WebSocket 流已结束，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                        };
                        
                        last_message_time = Instant::now();
                        
                        match msg {
                            Ok(Message::Text(text)) => {
                                message_count += 1;
                                
                                if message_count <= 10 || last_log_time.elapsed().as_secs() >= 10 {
                                    log::debug!("[OKX Futures] 收到消息 #{}: {}", message_count,
                                        if text.len() > 200 { &text[..200] } else { &text });
                                    last_log_time = Instant::now();
                                }
                                
                                // 解析消息
                                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(event) = json_msg.get("event").and_then(|v| v.as_str()) {
                                        if event == "subscribe" {
                                            log::info!("[OKX Futures] 📩 订阅成功");
                                        } else if event == "error" {
                                            log::error!("[OKX Futures] ❌ 订阅错误: {}", text);
                                        }
                                        continue;
                                    }
                                    
                                    // 处理数据消息
                                    if let Some(arg) = json_msg.get("arg") {
                                        if let Some(data_array) = json_msg.get("data").and_then(|v| v.as_array()) {
                                            let channel = arg.get("channel").and_then(|v| v.as_str()).unwrap_or("unknown");
                                            // OKX 的数据数组中每个元素都包含 instId
                                            for data_item in data_array {
                                                if let Some(inst_id) = data_item.get("instId").and_then(|v| v.as_str()) {
                                                    Self::handle_data(channel, inst_id, data_item, pulsar_client.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if message_count % 1000 == 0 {
                                    log::debug!("[OKX Futures] 已接收 {} 条消息", message_count);
                                }
                            }
                            Ok(Message::Pong(_)) => {
                                log::debug!("[OKX Futures] 💓 收到 Pong");
                            }
                            Ok(Message::Close(_)) => {
                                log::warn!("[OKX Futures] 收到 Close 帧，准备重连");
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            Err(e) => {
                                log::error!("[OKX Futures] WebSocket 错误: {}, 准备重连", e);
                                retry_count += 1;
                                continue 'reconnect;
                            }
                            _ => {}
                        }
                    }
                    
                    _ = &mut ctrl_c => {
                        log::info!("[OKX Futures] 收到关闭信号，停止 WebSocket");
                        break 'reconnect;
                    }
                }
            }
        }

        log::info!("[OKX Futures] WebSocket 守护任务已停止");
    }

    /// 处理数据消息
    fn handle_data(channel: &str, inst_id: &str, data: &serde_json::Value, _pulsar_client: Option<Arc<PulsarClient>>) {
        match channel {
            "tickers" => {
                log::debug!("[OKX Futures {}] Ticker - 原始数据: {:?}", inst_id, data);

                // 转换为 UnifiedTicker 并发送到 Pulsar
                match common::TickerConverter::from_okx_futures(data, inst_id) {
                    Ok(unified_ticker) => {
                        log::debug!(
                            "[OKX Futures {}] 转换成功 - 价格: {}, 涨跌幅: {:?}%", 
                            inst_id, unified_ticker.close, unified_ticker.change_percent_24h
                        );
                        common::PulsarClient::publish_async("futures-ticker", unified_ticker);
                    }
                    Err(e) => log::error!("[OKX Futures {}] Ticker 转换失败: {}", inst_id, e),
                }
            }
            "funding-rate" => {
                log::debug!("[OKX Futures {}] Funding Rate - 原始数据: {:?}", inst_id, data);
                
                // 转换为 UnifiedMarkPrice 并发送到 Pulsar
                match common::MarkPriceConverter::from_okx_funding_rate(data, inst_id) {
                    Ok(mark_price) => {
                        log::debug!(
                            "[OKX Futures {}] Funding Rate 转换成功 - 资金费率: {:?}, 结算时间: {:?}",
                            inst_id, mark_price.funding_rate, mark_price.funding_time
                        );
                        common::PulsarClient::publish_async(common::pulsar::mark_price::FUTURES_MARK_PRICE, mark_price);
                    }
                    Err(e) => log::error!("[OKX Futures {}] Funding Rate 转换失败: {}", inst_id, e),
                }
            }
            ch if ch.starts_with("candle") => {
                if let Some(kline_data) = data.as_array() {
                    if kline_data.len() >= 5 {
                        let close = kline_data.get(4).and_then(|v| v.as_str()).unwrap_or("0");
                        log::debug!("[OKX Futures {}] Kline {} - 收盘价: {}", inst_id, channel, close);
                    }
                }
            }
            ch if ch.starts_with("books") => {
                let asks = data.get("asks").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                let bids = data.get("bids").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                log::debug!("[OKX Futures {}] Depth - 买单: {}, 卖单: {}", inst_id, bids, asks);
            }
            _ => {}
        }
    }
}

impl Default for OkxFutures {
    fn default() -> Self {
        Self::new()
    }
}
