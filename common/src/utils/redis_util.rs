use deadpool_redis::{redis::cmd, Config, Pool, Runtime};
use crate::error::AppError;

/// Redis 工具类 - 封装 deadpool-redis 连接池
#[derive(Clone)]
pub struct RedisUtil {
    pool: Pool,
}

impl RedisUtil {
    /// 从 URL 创建 Redis 连接池
    pub fn from_url(url: String) -> Result<Self, AppError> {
        log::info!("Initializing Redis connection pool");

        // 创建连接池配置
        let cfg = Config::from_url(url);
        
        // 创建连接池
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Failed to create Redis pool: {}", e)})))?;

        log::info!("✅ Redis connection pool initialized successfully");

        Ok(RedisUtil { pool })
    }

    /// 获取连接池引用（用于注册到 Actix App Data）
    pub fn pool(&self) -> Pool {
        self.pool.clone()
    }

    /// SET - 设置键值
    pub async fn set(&self, key: &str, value: &str) -> Result<(), AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        cmd("SET")
            .arg(&[key, value])
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis SET error: {}", e)})))?;

        Ok(())
    }

    /// SETEX - 设置带过期时间的键值 (秒)
    pub async fn set_ex(&self, key: &str, value: &str, seconds: i64) -> Result<(), AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        cmd("SETEX")
            .arg(&[key, &seconds.to_string(), value])
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis SETEX error: {}", e)})))?;

        Ok(())
    }

    /// SET NX EX - 设置键值，如果键不存在并设置过期时间 (用于分布式锁)
    /// 返回 true 表示设置成功(获取锁成功)，false 表示键已存在
    /// 使用 SET key value NX EX seconds 实现原子操作
    pub async fn set_nx(&self, key: &str, value: &str, expire_seconds: i64) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        // 使用 SET key value NX EX seconds 实现原子操作
        // 如果成功返回 "OK"，如果键已存在返回 nil
        let result: Option<String> = cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(expire_seconds)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis SET NX EX error: {}", e)})))?;

        Ok(result.is_some())
    }

    /// GET - 获取值
    pub async fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let value: Option<String> = cmd("GET")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis GET error: {}", e)})))?;

        Ok(value)
    }

    /// DEL - 删除键
    pub async fn del(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let deleted: i32 = cmd("DEL")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis DEL error: {}", e)})))?;

        Ok(deleted > 0)
    }

    /// DEL_PATTERN - 删除匹配模式的所有键
    /// 使用 SCAN 命令查找匹配的键，然后批量删除
    /// 
    /// # Arguments
    /// * `pattern` - 匹配模式，例如 "app_config:*"
    /// 
    /// # Returns
    /// 返回删除的键数量
    pub async fn del_pattern(&self, pattern: &str) -> Result<i32, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let mut cursor: String = "0".to_string();
        let mut total_deleted = 0;
        
        loop {
            // 使用 SCAN 查找匹配的键
            let result: (String, Vec<String>) = cmd("SCAN")
                .arg(&cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg("100")
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis SCAN error: {}", e)})))?;
            
            cursor = result.0;
            let keys = result.1;
            
            // 批量删除找到的键
            if !keys.is_empty() {
                let mut del_cmd = cmd("DEL");
                for key in &keys {
                    del_cmd.arg(key);
                }
                
                let deleted: i32 = del_cmd
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis DEL error: {}", e)})))?;
                
                total_deleted += deleted;
                log::debug!("🗑️  Deleted {} keys matching pattern: {}", deleted, pattern);
            }
            
            // 如果 cursor 为 0，表示扫描完成
            if cursor == "0" {
                break;
            }
        }
        
        if total_deleted > 0 {
            log::info!("🗑️  Total deleted {} keys matching pattern: {}", total_deleted, pattern);
        } else {
            log::debug!("No keys found matching pattern: {}", pattern);
        }
        
        Ok(total_deleted)
    }

    /// EXISTS - 检查键是否存在
    pub async fn exists(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let exists: i32 = cmd("EXISTS")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis EXISTS error: {}", e)})))?;

        Ok(exists > 0)
    }

    /// EXPIRE - 设置过期时间 (秒)
    pub async fn expire(&self, key: &str, seconds: i64) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let set: i32 = cmd("EXPIRE")
            .arg(&[key, &seconds.to_string()])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis EXPIRE error: {}", e)})))?;

        Ok(set > 0)
    }

    /// TTL - 获取键的剩余生存时间 (秒)
    pub async fn ttl(&self, key: &str) -> Result<i64, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let ttl: i64 = cmd("TTL")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis TTL error: {}", e)})))?;

        Ok(ttl)
    }

    /// INCR - 自增
    pub async fn incr(&self, key: &str) -> Result<i64, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let value: i64 = cmd("INCR")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis INCR error: {}", e)})))?;

        Ok(value)
    }

    /// DECR - 自减
    pub async fn decr(&self, key: &str) -> Result<i64, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let value: i64 = cmd("DECR")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis DECR error: {}", e)})))?;

        Ok(value)
    }

    // ==================== Redis Stream Operations ====================

    /// XADD - 添加消息到 Stream
    /// 返回消息ID
    pub async fn xadd(
        &self,
        stream: &str,
        id: &str, // "*" 表示自动生成ID
        fields: &[(&str, &str)],
    ) -> Result<String, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let mut command = cmd("XADD");
        command.arg(stream).arg(id);
        
        for (key, value) in fields {
            command.arg(key).arg(value);
        }

        let message_id: String = command
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XADD error: {}", e)})))?;

        Ok(message_id)
    }

    /// XREAD - 读取 Stream 消息
    pub async fn xread(
        &self,
        stream: &str,
        id: &str, // 从哪个ID开始读取，"0" 表示从头开始，"$" 表示只读取新消息
        count: usize,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let result: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = cmd("XREAD")
            .arg("COUNT")
            .arg(count)
            .arg("STREAMS")
            .arg(stream)
            .arg(id)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XREAD error: {}", e)})))?;

        // 解析结果
        let messages = if let Some((_, stream_messages)) = result.first() {
            stream_messages.clone()
        } else {
            vec![]
        };

        Ok(messages)
    }

    /// XGROUP CREATE - 创建消费者组
    pub async fn xgroup_create(
        &self,
        stream: &str,
        group: &str,
        id: &str, // "0" 从头开始，"$" 从最新开始
    ) -> Result<(), AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let _: String = cmd("XGROUP")
            .arg("CREATE")
            .arg(stream)
            .arg(group)
            .arg(id)
            .arg("MKSTREAM") // 如果 stream 不存在则创建
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                // 忽略 "BUSYGROUP Consumer Group name already exists" 错误
                if e.to_string().contains("BUSYGROUP") {
                    return AppError::unknown("error.redis_Consumer group already exists");
                }
                AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XGROUP CREATE error: {}", e)}))
            })?;

        Ok(())
    }

    /// XREADGROUP - 消费者组读取消息
    pub async fn xreadgroup(
        &self,
        group: &str,
        consumer: &str,
        stream: &str,
        count: usize,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let result: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = cmd("XREADGROUP")
            .arg("GROUP")
            .arg(group)
            .arg(consumer)
            .arg("COUNT")
            .arg(count)
            .arg("STREAMS")
            .arg(stream)
            .arg(">") // ">" 表示只读取未被消费的新消息
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XREADGROUP error: {}", e)})))?;

        // 解析结果
        let messages = if let Some((_, stream_messages)) = result.first() {
            stream_messages.clone()
        } else {
            vec![]
        };

        Ok(messages)
    }

    /// XACK - 确认消息已处理
    pub async fn xack(&self, stream: &str, group: &str, message_id: &str) -> Result<i32, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let acked: i32 = cmd("XACK")
            .arg(stream)
            .arg(group)
            .arg(message_id)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XACK error: {}", e)})))?;

        Ok(acked)
    }

    /// XLEN - 获取 Stream 长度
    pub async fn xlen(&self, stream: &str) -> Result<i64, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let len: i64 = cmd("XLEN")
            .arg(stream)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XLEN error: {}", e)})))?;

        Ok(len)
    }

    /// XDEL - 删除 Stream 消息
    /// 返回删除的消息数量
    pub async fn xdel(&self, stream: &str, message_ids: &[&str]) -> Result<i32, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let mut command = cmd("XDEL");
        command.arg(stream);
        for id in message_ids {
            command.arg(id);
        }

        let deleted: i32 = command
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XDEL error: {}", e)})))?;

        Ok(deleted)
    }

    /// XTRIM - 修剪 Stream 到指定长度(保留最新的 N 条消息)
    /// 返回删除的消息数量
    pub async fn xtrim(&self, stream: &str, maxlen: i64) -> Result<i64, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let trimmed: i64 = cmd("XTRIM")
            .arg(stream)
            .arg("MAXLEN")
            .arg("~") // 近似修剪,性能更好
            .arg(maxlen)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis XTRIM error: {}", e)})))?;

        Ok(trimmed)
    }

    // ==================== Redis Hash Operations ====================

    /// HGET - 获取 Hash 字段值
    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let value: Option<String> = cmd("HGET")
            .arg(key)
            .arg(field)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis HGET error: {}", e)})))?;

        Ok(value)
    }

    /// HSET - 设置 Hash 字段值
    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let set: i32 = cmd("HSET")
            .arg(key)
            .arg(field)
            .arg(value)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis HSET error: {}", e)})))?;

        Ok(set > 0)
    }

    /// HEXISTS - 检查 Hash 字段是否存在
    pub async fn hexists(&self, key: &str, field: &str) -> Result<bool, AppError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis connection error: {}", e)})))?;

        let exists: i32 = cmd("HEXISTS")
            .arg(key)
            .arg(field)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::unknown_with_params("error.redis", serde_json::json!({"msg": format!("Redis HEXISTS error: {}", e)})))?;

        Ok(exists > 0)
    }
}

