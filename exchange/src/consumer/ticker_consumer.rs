use common::pulsar::{PulsarClient, topics};
use common::models::UnifiedTicker;
use common::UnifiedMarkPrice;
use futures::StreamExt;
use pulsar::Consumer;
use crate::config::disruptor_config::publish as disruptor_publish;
use crate::common::mark_price_cache;

/// 启动 Ticker 消费者
pub async fn start_ticker_consumer() {
    log::info!("正在启动 Ticker 消费者...");

    let Some(client) = PulsarClient::global() else {
        log::error!("PulsarClient 未初始化，无法启动消费者");
        return;
    };

    // 1. 订阅 Spot Ticker
    spawn_ticker_consumer(&client, topics::ticker::SPOT_TICKER, "exchange-spot-ticker-group", "Spot").await;

    // 2. 订阅 Futures Ticker
    spawn_ticker_consumer(&client, topics::ticker::FUTURES_TICKER, "exchange-futures-ticker-group", "Futures").await;

    // 3. 订阅 Futures MarkPrice
    spawn_mark_price_consumer(&client, topics::mark_price::FUTURES_MARK_PRICE, "exchange-futures-mark-price-group").await;
}

/// 创建并启动 Ticker 消费者
async fn spawn_ticker_consumer(client: &PulsarClient, topic: &str, subscription: &str, label: &'static str) {
    match client.subscribe::<UnifiedTicker>(topic, subscription).await {
        Ok(consumer) => {
            log::info!("✅ {} Ticker 消费者已启动", label);
            tokio::spawn(consume_ticker_loop(consumer, label));
        }
        Err(e) => log::error!("订阅 {} Ticker 失败: {}", label, e),
    }
}

/// Ticker 消费循环
async fn consume_ticker_loop(mut consumer: Consumer<UnifiedTicker, pulsar::TokioExecutor>, label: &'static str) {
    while let Some(msg) = consumer.next().await {
        let Ok(msg) = msg else {
            log::error!("[{}] 接收消息失败: {:?}", label, msg.err());
            continue;
        };

        match msg.deserialize() {
            Ok(ticker) => {
                log::debug!("[{}] 收到 Ticker: {} {} Price: {}", 
                    label, ticker.exchange, ticker.symbol, ticker.close);

                // 推送到 Disruptor 处理
                disruptor_publish(|e| e.ticker = ticker.clone());

                // 确认消息
                if let Err(e) = consumer.ack(&msg).await {
                    log::error!("[{}] Ack 失败: {}", label, e);
                }
            }
            Err(e) => log::error!("[{}] 反序列化 Ticker 失败: {}", label, e),
        }
    }
    log::warn!("{} Ticker 消费者循环退出", label);
}

/// 创建并启动 MarkPrice 消费者
async fn spawn_mark_price_consumer(client: &PulsarClient, topic: &str, subscription: &str) {
    match client.subscribe::<UnifiedMarkPrice>(topic, subscription).await {
        Ok(consumer) => {
            log::info!("✅ Futures MarkPrice 消费者已启动");
            tokio::spawn(consume_mark_price_loop(consumer));
        }
        Err(e) => log::error!("订阅 Futures MarkPrice 失败: {}", e),
    }
}

/// MarkPrice 消费循环
async fn consume_mark_price_loop(mut consumer: Consumer<UnifiedMarkPrice, pulsar::TokioExecutor>) {
    while let Some(msg) = consumer.next().await {
        let Ok(msg) = msg else {
            log::error!("[MarkPrice] 接收消息失败: {:?}", msg.err());
            continue;
        };

        match msg.deserialize() {
            Ok(mark_price) => {
                log::debug!("[MarkPrice] 收到: {} {} MarkPrice: {} IndexPrice: {}",
                    mark_price.exchange, mark_price.symbol, 
                    mark_price.mark_price, mark_price.index_price);

                // 存储到缓存并更新聚合值
                mark_price_cache::set_mark_price(mark_price);

                // 确认消息
                if let Err(e) = consumer.ack(&msg).await {
                    log::error!("[MarkPrice] Ack 失败: {}", e);
                }
            }
            Err(e) => log::error!("[MarkPrice] 反序列化失败: {}", e),
        }
    }
    log::warn!("Futures MarkPrice 消费者循环退出");
}
