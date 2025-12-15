use crate::mq::message_queue::MessageQueue;
use crate::mq::subscriber_trait::MessageSubscriber;

pub mod message_queue;
pub mod subscriber_trait;

/// 辅助函数：注册单个订阅者
pub async fn register_subscriber<T: MessageSubscriber + Clone + 'static>(
    mq: &MessageQueue,
    subscriber: T,
) {
    let topic = subscriber.topic().to_string();

    mq.subscribe(&topic, move |msg| {
        let sub = subscriber.clone();
        Box::pin(async move {
            sub.handle(msg).await
        })
    }).await;

    log::info!("   ✅ Registered subscriber for topic: '{}'", topic);
}
