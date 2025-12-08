use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, OnceCell};

use futures::StreamExt;
use pulsar::{
    Consumer, DeserializeMessage, Pulsar, SubType, TokioExecutor, ProducerOptions,
};
use serde::{Deserialize, Serialize};

/// 泛型事件结构要求实现序列化（用于 JSON）
pub trait Event: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {}

impl<T> Event for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {}

/// 全局 PulsarClient 单例
static GLOBAL_PULSAR_CLIENT: OnceCell<Arc<PulsarClient>> = OnceCell::const_new();

/// Pulsar 客户端封装
/// 支持多个 Producer，按 topic 自动管理
pub struct PulsarClient {
    client: Arc<RwLock<Option<Pulsar<TokioExecutor>>>>,
    producers: Arc<RwLock<HashMap<String, pulsar::Producer<TokioExecutor>>>>,
}

impl PulsarClient {
    /// 创建未初始化的 PulsarClient
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            producers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 初始化全局 PulsarClient
    pub async fn init_global(url: &str) -> Result<(), pulsar::Error> {
        let client = Arc::new(Self::new());
        client.connect(url).await?;
        
        GLOBAL_PULSAR_CLIENT.set(client.clone())
            .map_err(|_| pulsar::Error::Custom("Global PulsarClient already initialized".to_string()))?;
        
        log::info!("✅ 全局 PulsarClient 已初始化");
        Ok(())
    }

    /// 获取全局 PulsarClient 实例
    pub fn global() -> Option<Arc<PulsarClient>> {
        GLOBAL_PULSAR_CLIENT.get().cloned()
    }

    // ==================== 静态便捷方法 ====================

    /// 发送消息（静态方法，自动使用全局实例）
    /// 
    /// # 示例
    /// ```ignore
    /// PulsarClient::publish("my-topic", &data).await;
    /// ```
    pub async fn publish<T: Event>(topic: &str, msg: &T) {
        let Some(client) = Self::global() else {
            log::warn!("[Pulsar] 未初始化，跳过发送");
            return;
        };
        if let Err(e) = client.send(topic, msg).await {
            log::error!("[Pulsar] 发送失败: {} - {}", topic, e);
        }
    }

    /// 发送消息（静态方法，异步后台执行，不阻塞当前线程）
    /// 
    /// # 示例
    /// ```ignore
    /// PulsarClient::publish_async("my-topic", data);
    /// ```
    pub fn publish_async<T: Event + Clone>(topic: &'static str, msg: T) {
        tokio::spawn(async move {
            Self::publish(topic, &msg).await;
        });
    }

    /// 发送消息并等待确认（静态方法）
    pub async fn publish_blocking<T: Event>(topic: &str, msg: &T) -> Result<(), pulsar::Error> {
        let client = Self::global()
            .ok_or_else(|| pulsar::Error::Custom("PulsarClient 未初始化".to_string()))?;
        client.send_blocking(topic, msg).await
    }

    /// 连接到 Pulsar 服务器并初始化客户端
    pub async fn connect(&self, url: &str) -> Result<(), pulsar::Error> {
        let pulsar_client: Pulsar<_> = Pulsar::builder(url, TokioExecutor).build().await?;
        let mut client = self.client.write().await;
        *client = Some(pulsar_client);
        log::info!("✅ PulsarClient 已成功连接到: {}", url);
        Ok(())
    }

    /// 检查客户端是否已初始化
    async fn ensure_initialized(&self) -> Result<(), pulsar::Error> {
        let client = self.client.read().await;
        if client.is_none() {
            log::error!("❌ PulsarClient 未初始化！请先调用 connect() 方法");
            return Err(pulsar::Error::Custom("PulsarClient not initialized".to_string()));
        }
        Ok(())
    }

    /// 获取或创建指定 topic 的 Producer
    async fn get_or_create_producer(&self, topic: &str) -> Result<(), pulsar::Error> {
        // 先检查是否已存在
        {
            let producers = self.producers.read().await;
            if producers.contains_key(topic) {
                return Ok(());
            }
        }

        // 不存在则创建
        self.ensure_initialized().await?;
        
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().unwrap();
        
        let new_producer = client
            .producer()
            .with_topic(topic)
            .with_options(ProducerOptions::default())
            .build()
            .await?;
        
        let mut producers = self.producers.write().await;
        producers.insert(topic.to_string(), new_producer);
        
        log::info!("✅ Producer 已创建，topic: {}", topic);
        Ok(())
    }

    /// 订阅指定 Topic
    pub async fn subscribe<T: DeserializeMessage>(&self, topic: &str, subscription: &str) -> Result<Consumer<T, TokioExecutor>, pulsar::Error> {
        self.ensure_initialized().await?;
        
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().unwrap();
        
        let consumer = client
            .consumer()
            .with_topic(topic)
            .with_subscription(subscription)
            .with_subscription_type(SubType::Shared)
            .build()
            .await?;
            
        log::info!("✅ 已订阅 Topic: {}, Subscription: {}", topic, subscription);
        Ok(consumer)
    }

    /// 发送消息到指定 topic（异步非阻塞，不等待确认）
    /// 
    /// # 参数
    /// * `topic` - 目标 topic
    /// * `msg` - 要发送的消息
    /// 
    /// # 示例
    /// ```ignore
    /// PulsarClient::global().unwrap().send("my-topic", &my_data).await?;
    /// ```
    pub async fn send<T: Event>(
        &self,
        topic: &str,
        msg: &T,
    ) -> Result<(), pulsar::Error> {
        // 确保 Producer 存在
        self.get_or_create_producer(topic).await?;
        
        let payload = serde_json::to_vec(msg)
            .map_err(|e| pulsar::Error::Custom(format!("JSON序列化失败: {}", e)))?;
        
        let mut producers = self.producers.write().await;
        let producer = producers.get_mut(topic).unwrap();
        producer.send_non_blocking(payload).await?;
        Ok(())
    }

    /// 发送消息到指定 topic（同步阻塞，等待发送确认）
    pub async fn send_blocking<T: Event>(
        &self,
        topic: &str,
        msg: &T,
    ) -> Result<(), pulsar::Error> {
        // 确保 Producer 存在
        self.get_or_create_producer(topic).await?;
        
        let payload = serde_json::to_vec(msg)
            .map_err(|e| pulsar::Error::Custom(format!("JSON序列化失败: {}", e)))?;
        
        let mut producers = self.producers.write().await;
        let producer = producers.get_mut(topic).unwrap();
        producer.send_non_blocking(payload).await?.await?;
        Ok(())
    }

    /// 发送延时消息到指定 topic（异步非阻塞，不等待确认）
    pub async fn send_delay<T: Event>(
        &self,
        topic: &str,
        msg: &T,
        delay_sec: u64,
    ) -> Result<(), pulsar::Error> {
        // 确保 Producer 存在
        self.get_or_create_producer(topic).await?;
        
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let payload = serde_json::to_vec(msg)
            .map_err(|e| pulsar::Error::Custom(format!("JSON序列化失败: {}", e)))?;
        
        // 计算延时后的时间戳（毫秒）
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| pulsar::Error::Custom(format!("获取系统时间失败: {}", e)))?;
        
        let deliver_at_ms = (now.as_millis() as i64) + (delay_sec as i64 * 1000);
        
        // 创建带延时的消息
        let message = pulsar::producer::Message {
            payload,
            deliver_at_time: Some(deliver_at_ms),
            ..Default::default()
        };
        
        let mut producers = self.producers.write().await;
        let producer = producers.get_mut(topic).unwrap();
        producer.send_non_blocking(message).await?;
        log::info!("✅ 延时消息已发送到 topic: {}, 将在 {} 秒后投递", topic, delay_sec);
        Ok(())
    }

    /// 创建 Consumer（Shared）
    pub async fn create_consumer<T>(
        &self,
        topic: &str,
        sub_name: &str,
    ) -> Result<Consumer<T, TokioExecutor>, pulsar::Error>
    where
        T: DeserializeMessage + Event,
    {
        self.ensure_initialized().await?;
        
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().unwrap();
        
        let consumer = client
            .consumer()
            .with_topic(topic)
            .with_subscription(sub_name)
            .with_subscription_type(SubType::Shared)
            .build::<T>()
            .await?;

        log::info!("✅ Consumer 已创建，topic: {}, subscription: {}", topic, sub_name);
        Ok(consumer)
    }

    /// 消费消息（自动 ACK）
    pub async fn consume_loop<T, F>(mut consumer: Consumer<T, TokioExecutor>, mut handler: F)
    where
        T: Event + DeserializeMessage<Output = T>,
        F: FnMut(T) + Send + 'static,
    {
        log::info!("🔄 Consumer 开始运行...");

        while let Some(msg) = consumer.next().await {
            match msg {
                Ok(message) => {
                    let event = message.deserialize();
                    handler(event);
                    if let Err(e) = consumer.ack(&message).await {
                        log::error!("❌ ACK 失败: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("❌ 消费消息错误: {:?}", e);
                }
            }
        }
    }
}

impl Default for PulsarClient {
    fn default() -> Self {
        Self::new()
    }
}
