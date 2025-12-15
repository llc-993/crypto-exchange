use crate::error::AppError;
use crate::utils::redis_util::RedisUtil;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::pin::Pin;
use std::future::Future;

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T = serde_json::Value> {
    pub id: Option<String>,
    pub topic: String,
    pub payload: T,
    pub timestamp: i64,
}

impl<T> Message<T> {
    pub fn new(topic: impl Into<String>, payload: T) -> Self {
        Message {
            id: None,
            topic: topic.into(),
            payload,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// 消息处理器类型 - 接收消息并返回 Future
/// 默认使用 serde_json::Value 作为 payload 类型
pub type MessageHandler = Arc<dyn Fn(Message<serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>> + Send + Sync>;

/// 订阅者信息
struct Subscriber {
    topic: String,
    handler: MessageHandler,
}

/// 消息队列 - 基于 Redis Stream（支持发布-订阅模式）
#[derive(Clone)]
pub struct MessageQueue {
    redis: Arc<RedisUtil>,
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    consumer_group: String,
    /// 是否自动删除已确认的消息(默认: false,保留消息历史)
    auto_delete_messages: bool,
}

impl MessageQueue {
    pub fn new(redis: Arc<RedisUtil>) -> Self {
        MessageQueue {
            redis,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            consumer_group: "default-group".to_string(),
            auto_delete_messages: true, // 默认自动删除
        }
    }
    
    
    /// 订阅特定主题的消息
    /// 
    /// # 示例
    /// ```
    /// mq.subscribe("order.created", |msg| {
    ///     Box::pin(async move {
    ///         println!("Processing order: {:?}", msg);
    ///         Ok(())
    ///     })
    /// }).await;
    /// ```
    pub async fn subscribe<F>(&self, topic: impl Into<String>, handler: F)
    where
        F: Fn(Message<serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>> + Send + Sync + 'static,
    {
        let topic = topic.into();
        log::info!("📌 Subscribing to topic: '{}'", topic);

        let subscriber = Subscriber {
            topic: topic.clone(),
            handler: Arc::new(handler),
        };

        self.subscribers.write().await.push(subscriber);
        log::info!("✅ Successfully subscribed to topic: '{}'", topic);
    }

    /// 启动后台消费者（自动处理订阅的消息）
    ///
    /// 此方法会根据已订阅的主题自动创建对应的 stream 并启动消费者
    /// stream 名称格式：mq:{topic}
    pub async fn start_consumer(&self) -> Result<(), AppError> {
        let redis = self.redis.clone();
        let subscribers = self.subscribers.clone();
        let group = self.consumer_group.clone();
        let consumer_name = format!("consumer-{}", uuid::Uuid::new_v4());
        let auto_delete = self.auto_delete_messages; // 获取配置

        // 收集所有已订阅的主题
        let topics: Vec<String> = {
            let subs = subscribers.read().await;
            subs.iter()
                .filter(|s| s.topic != "*") // 排除通配符订阅
                .map(|s| s.topic.clone())
                .collect()
        };

        if topics.is_empty() {
            log::warn!("⚠️  No topics subscribed, consumer will not start");
            return Ok(());
        }

        log::info!("🚀 Starting background consumer for topics: {:?}", topics);
        if auto_delete {
            log::info!("🗑️  Auto-delete mode enabled: messages will be deleted after acknowledgment");
        } else {
            log::info!("📚 Message history mode: messages will be kept after acknowledgment");
        }

        // 为每个主题创建消费者组
        for topic in &topics {
            let stream = format!("mq:{}", topic);
            self.create_consumer_group(&stream, &group, "0").await.ok();
        }

        // 启动后台任务
        tokio::spawn(async move {
            log::info!("👂 Consumer '{}' is listening on topics: {:?}", consumer_name, topics);

            loop {
                // 遍历所有主题的 stream
                for topic in &topics {
                    let stream = format!("mq:{}", topic);

                    // 读取消息
                    match redis.xreadgroup(&group, &consumer_name, &stream, 10).await {
                        Ok(messages) => {
                            if !messages.is_empty() {
                                log::debug!("📬 Received {} messages from topic '{}'", messages.len(), topic);
                            }

                            for (message_id, fields) in messages {
                                match Self::parse_message_static(&message_id, &fields) {
                                    Ok(message) => {
                                        let msg_topic = message.topic.clone();

                                        // 查找匹配的订阅者
                                        let handlers = {
                                            let subs = subscribers.read().await;
                                            subs.iter()
                                                .filter(|s| s.topic == msg_topic || s.topic == "*")
                                                .map(|s| s.handler.clone())
                                                .collect::<Vec<_>>()
                                        };

                                        if handlers.is_empty() {
                                            log::warn!("⚠️  No subscriber for topic: '{}', message will be acknowledged anyway", msg_topic);
                                            // 没有订阅者,仍然确认消息避免重复处理
                                            if let Err(e) = redis.xack(&stream, &group, &message_id).await {
                                                log::error!("❌ Failed to ACK message {}: {}", message_id, e);
                                            } else {
                                                log::debug!("✓ Message {} acknowledged (no subscribers)", message_id);
                                            }
                                        } else {
                                            // 调用所有匹配的处理器
                                            let mut all_success = true;
                                            let mut success_count = 0;
                                            let mut error_count = 0;

                                            for handler in handlers {
                                                match handler(message.clone()).await {
                                                    Ok(_) => {
                                                        success_count += 1;
                                                        log::debug!("✅ Handler processed message {} successfully", message_id);
                                                    }
                                                    Err(e) => {
                                                        error_count += 1;
                                                        all_success = false;
                                                        log::error!("❌ Handler failed to process message {}: {}", message_id, e);
                                                    }
                                                }
                                            }

                                            // 只有所有处理器都成功时才确认消息
                                            if all_success {
                                                // 确认消息
                                                if let Err(e) = redis.xack(&stream, &group, &message_id).await {
                                                    log::error!("❌ Failed to ACK message {}: {}", message_id, e);
                                                } else {
                                                    log::info!("✓ Message {} acknowledged (topic: '{}', {} handlers succeeded)", 
                                                        message_id, msg_topic, success_count);
                                                    
                                                    // 如果配置了自动删除,则删除消息
                                                    if auto_delete {
                                                        match redis.xdel(&stream, &[&message_id]).await {
                                                            Ok(deleted) if deleted > 0 => {
                                                                log::debug!("🗑️  Message {} deleted from stream", message_id);
                                                            }
                                                            Ok(_) => {
                                                                log::warn!("⚠️  Message {} not found for deletion", message_id);
                                                            }
                                                            Err(e) => {
                                                                log::error!("❌ Failed to delete message {}: {}", message_id, e);
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                log::error!("⚠️  Message {} NOT acknowledged due to handler failures ({} succeeded, {} failed)", 
                                                    message_id, success_count, error_count);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("❌ Failed to parse message {}: {}", message_id, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Failed to read from stream '{}': {}", stream, e);
                        }
                    }
                }

                // 短暂延迟避免CPU占用过高
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        Ok(())
    }

    /// 发布消息到队列
    ///
    /// 消息会自动发布到对应主题的 stream，格式：mq:{topic}
    pub async fn publish<T: Serialize + Sync>(&self, message: &Message<T>) -> Result<String, AppError> {
        let stream = format!("mq:{}", message.topic);
        log::debug!("📤 Publishing message to stream: {} (topic: '{}')", stream, message.topic);

        // 准备 payload JSON
        let payload_json = serde_json::to_string(&message.payload)
            .map_err(|e| AppError::unknown_with_params("error.internal", serde_json::json!({"msg": format!("Failed to serialize payload: {}", e)})))?;

        // 准备时间戳字符串
        let timestamp_str = message.timestamp.to_string();

        // 准备字段
        let fields = vec![
            ("topic", message.topic.as_str()),
            ("timestamp", timestamp_str.as_str()),
            ("payload", payload_json.as_str()),
        ];

        // 添加到 Stream
        let message_id = self.redis.xadd(&stream, "*", &fields).await?;

        log::info!("✅ Message published to topic '{}' with ID: {}", message.topic, message_id);

        Ok(message_id)
    }

    /// 消费消息（简单模式 - 用于手动拉取）
    pub async fn consume_simple(&self, stream: &str, last_id: &str, count: usize) -> Result<Vec<(String, Message<serde_json::Value>)>, AppError> {
        log::debug!("📥 Reading messages from stream: {} (last_id: {})", stream, last_id);

        let raw_messages = self.redis.xread(stream, last_id, count).await?;

        let mut messages = Vec::new();
        for (id, fields) in raw_messages {
            if let Ok(message) = Self::parse_message_static(&id, &fields) {
                messages.push((id, message));
            }
        }

        log::info!("📬 Retrieved {} messages from stream '{}'", messages.len(), stream);

        Ok(messages)
    }

    /// 创建消费者组
    pub async fn create_consumer_group(&self, stream: &str, group: &str, start_id: &str) -> Result<(), AppError> {
        log::debug!("👥 Creating consumer group '{}' for stream '{}'", group, stream);

        match self.redis.xgroup_create(stream, group, start_id).await {
            Ok(_) => {
                log::info!("✅ Consumer group '{}' created successfully", group);
                Ok(())
            }
            Err(e) => {
                if e.to_string().contains("already exists") {
                    log::debug!("⚠️  Consumer group '{}' already exists", group);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 消费消息（消费者组模式 - 用于手动拉取）
    pub async fn consume_group(
        &self,
        stream: &str,
        group: &str,
        consumer: &str,
        count: usize,
    ) -> Result<Vec<(String, Message<serde_json::Value>)>, AppError> {
        log::debug!(
            "📥 Reading messages from stream: {} (group: {}, consumer: {})",
            stream, group, consumer
        );

        let raw_messages = self.redis.xreadgroup(group, consumer, stream, count).await?;

        let mut messages = Vec::new();
        for (id, fields) in raw_messages {
            if let Ok(message) = Self::parse_message_static(&id, &fields) {
                messages.push((id, message));
            }
        }

        log::info!(
            "📬 Consumer '{}' retrieved {} messages from stream '{}'",
            consumer, messages.len(), stream
        );

        Ok(messages)
    }

    /// 确认消息已处理
    pub async fn ack(&self, stream: &str, group: &str, message_id: &str) -> Result<(), AppError> {
        log::debug!("✓ Acknowledging message {} in group '{}'", message_id, group);

        let acked = self.redis.xack(stream, group, message_id).await?;

        if acked > 0 {
            log::debug!("✅ Message {} acknowledged", message_id);
        } else {
            log::warn!("⚠️  Message {} was already acknowledged or doesn't exist", message_id);
        }

        Ok(())
    }

    /// 获取队列长度
    pub async fn len(&self, stream: &str) -> Result<i64, AppError> {
        self.redis.xlen(stream).await
    }

    /// 解析消息（静态方法）
    fn parse_message_static(id: &str, fields: &[(String, String)]) -> Result<Message<serde_json::Value>, AppError> {
        let mut topic = String::new();
        let mut timestamp: i64 = 0;
        let mut payload = serde_json::Value::Null;

        for (key, value) in fields {
            match key.as_str() {
                "topic" => topic = value.clone(),
                "timestamp" => {
                    timestamp = value.parse().unwrap_or(0);
                }
                "payload" => {
                    payload = serde_json::from_str(value)
                        .map_err(|e| AppError::unknown_with_params("error.internal_error", serde_json::json!({"msg": format!("Failed to parse payload: {}", e)})))?;
                }
                _ => {}
            }
        }

        Ok(Message {
            id: Some(id.to_string()),
            topic,
            payload,
            timestamp,
        })
    }
}
