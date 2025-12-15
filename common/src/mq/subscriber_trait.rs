use crate::error::AppError;
use crate::mq::message_queue::Message;
use async_trait::async_trait;

/// 消息订阅者 Trait
#[async_trait]
pub trait MessageSubscriber: Send + Sync {
    /// 订阅的主题
    fn topic(&self) -> &str;

    /// 处理消息
    async fn handle(&self, message: Message) -> Result<(), AppError>;
}