use common::models::coin_thumb_price::CoinThumbPrice;
use common::pulsar::topics;
use common::PulsarClient;
use futures::StreamExt;
use pulsar::Consumer;
use common::enums::KlineInterval;
use crate::handle::kline_handle;


pub async fn start_thumb_price_consumer() {
    log::info!("正在启动价格消费者...");

    let Some(client) = PulsarClient::global() else {
        log::error!("PulsarClient 未初始化，无法启动消费者");
        return;
    };

    match client.subscribe::<CoinThumbPrice>(topics::ticker::EX_THUMB_PRICE, "ex-thumb-price-group").await {
        Ok(consumer) => {
            log::info!("✅ 价格消费者已启动");
            tokio::spawn(consume_thumb_price_loop(consumer));
        }
        Err(e) => log::error!("订阅价格失败: {}", e),
    }
}

async fn consume_thumb_price_loop(mut consumer: Consumer<CoinThumbPrice, pulsar::TokioExecutor>) {
    while let Some(msg) = consumer.next().await {
        let Ok(msg) = msg else {
            log::error!("[价格] 接收消息失败: {:?}", msg.err());
            continue;
        };

        match msg.deserialize() {
            Ok(thumb_price) => {
                log::debug!("[价格] 收到: {} {:?} 价格: {}",
                    thumb_price.symbol, thumb_price.market_type, thumb_price.price);

                // TODO: 处理价格数据（存储到缓存、生成K线等）
                // 1、生成k线
                // 生成所有时间间隔的K线
                kline_handle::generate_all_klines(&thumb_price).await;


                if let Err(e) = consumer.ack(&msg).await {
                    log::error!("[价格] Ack 失败: {}", e);
                }
            }
            Err(e) => log::error!("[价格] 反序列化失败: {}", e),
        }
    }
    log::warn!("价格消费者循环退出");
}