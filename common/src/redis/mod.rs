mod redis_cache;
pub mod redis_key;

pub use redis_cache::RedisCache;
pub use redis_key as keys;

// Redis连接模块
use redis::{Client, RedisError, RedisResult};
use redis::aio::ConnectionManager;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

// 重新导出配置
pub use crate::config::redis_conf::RedisConfig;

/// 全局 RedisCache 实例
static GLOBAL_REDIS_CACHE: OnceCell<Mutex<RedisCache>> = OnceCell::new();

/// 创建 Redis 客户端
pub fn create_client(redis_url: &str) -> Result<Client, RedisError> {
    redis::Client::open(redis_url)
}

/// 创建异步连接管理器（推荐用于生产环境）
/// 
/// ConnectionManager 会自动重连，适合长期运行的应用
pub async fn create_async_connection(redis_url: &str) -> Result<ConnectionManager, RedisError> {
    let client = create_client(redis_url)?;
    ConnectionManager::new(client).await
}

/// 从配置创建异步连接管理器，并初始化全局 RedisCache
pub async fn create_async_connection_from_config(config: &RedisConfig) -> Result<ConnectionManager, RedisError> {
    let conn = create_async_connection(&config.build_url()).await?;
    
    // 初始化全局 RedisCache（创建新的连接用于全局缓存）
    let redis_cache = RedisCache::new(conn.clone());
    GLOBAL_REDIS_CACHE.set(Mutex::new(redis_cache))
        .map_err(|_| RedisError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "RedisCache 只能初始化一次",
        )))?;
    log::info!("✅ 全局 RedisCache 初始化成功");
    
    Ok(conn.clone())
}

/// 获取全局 RedisCache 实例
fn get_redis_cache() -> &'static Mutex<RedisCache> {
    GLOBAL_REDIS_CACHE.get()
        .expect("RedisCache 未初始化，请先调用 create_async_connection_from_config")
}

/// 测试 Redis 连接
pub async fn test_connection(conn: &mut ConnectionManager) -> Result<bool, RedisError> {
    let pong: String = redis::cmd("PING").query_async(conn).await?;
    if pong == "PONG" {
        log::info!("✅ Redis 连接测试成功");
        Ok(true)
    } else {
        log::error!("❌ Redis 连接测试失败");
        Err(RedisError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Redis连接测试失败",
        )))
    }
}

/// 全局 RedisCache 便捷访问对象
/// 
/// 使用方式：
/// ```rust
/// use common::cache;
/// 
/// // 在 main 函数中初始化（调用 create_async_connection_from_config 时会自动初始化）
/// let redis_config = common::RedisConfig::from_url("redis://localhost:6379", 10);
/// let _conn = common::create_async_connection_from_config(&redis_config).await?;
/// 
/// // 然后就可以直接使用了
/// cache.set("key", "value").await?;
/// let value = cache.get("key").await?;
/// ```
pub struct Cache;

impl Cache {
    /// 设置字符串值
    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        get_redis_cache().lock().await.set(key, value).await
    }

    /// 设置字符串值（带过期时间）
    pub async fn setex(&self, key: &str, value: &str, seconds: usize) -> RedisResult<()> {
        get_redis_cache().lock().await.setex(key, value, seconds).await
    }

    /// 获取字符串值
    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        get_redis_cache().lock().await.get(key).await
    }

    /// 删除键
    pub async fn del(&self, keys: &[&str]) -> RedisResult<usize> {
        get_redis_cache().lock().await.del(keys).await
    }

    /// 检查键是否存在
    pub async fn exists(&self, keys: &[&str]) -> RedisResult<usize> {
        get_redis_cache().lock().await.exists(keys).await
    }

    /// 设置过期时间
    pub async fn expire(&self, key: &str, seconds: usize) -> RedisResult<bool> {
        get_redis_cache().lock().await.expire(key, seconds).await
    }

    /// Hash: 设置字段值
    pub async fn hset(&self, key: &str, field: &str, value: &str) -> RedisResult<bool> {
        get_redis_cache().lock().await.hset(key, field, value).await
    }

    /// Hash: 获取字段值
    pub async fn hget(&self, key: &str, field: &str) -> RedisResult<Option<String>> {
        get_redis_cache().lock().await.hget(key, field).await
    }

    /// Hash: 获取所有字段和值
    pub async fn hgetall(&self, key: &str) -> RedisResult<Vec<(String, String)>> {
        get_redis_cache().lock().await.hgetall(key).await
    }

    /// Set: 添加成员
    pub async fn sadd(&self, key: &str, members: &[&str]) -> RedisResult<usize> {
        get_redis_cache().lock().await.sadd(key, members).await
    }

    /// Set: 获取所有成员
    pub async fn smembers(&self, key: &str) -> RedisResult<std::collections::HashSet<String>> {
        get_redis_cache().lock().await.smembers(key).await
    }

    /// ZSet: 添加成员
    pub async fn zadd(&self, key: &str, score: f64, member: &str) -> RedisResult<bool> {
        get_redis_cache().lock().await.zadd(key, score, member).await
    }

    /// ZSet: 按排名获取
    pub async fn zrange(&self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<String>> {
        get_redis_cache().lock().await.zrange(key, start, stop).await
    }

    /// List: 左侧推入
    pub async fn lpush(&self, key: &str, values: &[&str]) -> RedisResult<usize> {
        get_redis_cache().lock().await.lpush(key, values).await
    }

    /// List: 右侧推入
    pub async fn rpush(&self, key: &str, values: &[&str]) -> RedisResult<usize> {
        get_redis_cache().lock().await.rpush(key, values).await
    }

    /// List: 左侧弹出
    pub async fn lpop(&self, key: &str) -> RedisResult<Option<String>> {
        get_redis_cache().lock().await.lpop(key).await
    }

    /// List: 获取范围
    pub async fn lrange(&self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<String>> {
        get_redis_cache().lock().await.lrange(key, start, stop).await
    }
}

/// 全局 cache 实例
#[allow(non_upper_case_globals)]
pub static cache: Cache = Cache;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let result = create_client("redis://localhost:6379");
        assert!(result.is_ok());
    }
}