//! Redis 缓存操作工具类
//! 
//! 提供 String、Hash、Set、ZSet、List 等数据类型的完整操作接口

use redis::aio::ConnectionManager;
use redis::RedisResult;
use std::collections::HashSet;

/// Redis 缓存操作工具类
#[derive(Clone)]
pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    /// 创建新的 RedisCache 实例
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    // ==================== String 操作 ====================

    /// 设置字符串值
    pub async fn set(&mut self, key: &str, value: &str) -> RedisResult<()> {
        redis::cmd("SET").arg(key).arg(value).query_async(&mut self.conn).await
    }

    /// 设置字符串值（带过期时间，单位：秒）
    pub async fn setex(&mut self, key: &str, value: &str, seconds: usize) -> RedisResult<()> {
        redis::cmd("SETEX").arg(key).arg(seconds).arg(value).query_async(&mut self.conn).await
    }

    /// 设置字符串值（带过期时间，单位：毫秒）
    pub async fn psetex(&mut self, key: &str, value: &str, milliseconds: usize) -> RedisResult<()> {
        redis::cmd("PSETEX").arg(key).arg(milliseconds).arg(value).query_async(&mut self.conn).await
    }

    /// 获取字符串值
    pub async fn get(&mut self, key: &str) -> RedisResult<Option<String>> {
        redis::cmd("GET").arg(key).query_async(&mut self.conn).await
    }

    /// 获取多个键的值
    pub async fn mget(&mut self, keys: &[&str]) -> RedisResult<Vec<Option<String>>> {
        let mut cmd = redis::cmd("MGET");
        for key in keys {
            cmd.arg(key);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 设置多个键值对
    pub async fn mset(&mut self, pairs: &[(&str, &str)]) -> RedisResult<()> {
        let mut cmd = redis::cmd("MSET");
        for (key, value) in pairs {
            cmd.arg(key).arg(value);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 如果键不存在则设置
    pub async fn setnx(&mut self, key: &str, value: &str) -> RedisResult<bool> {
        redis::cmd("SETNX").arg(key).arg(value).query_async(&mut self.conn).await
    }

    /// 获取并设置值
    pub async fn getset(&mut self, key: &str, value: &str) -> RedisResult<Option<String>> {
        redis::cmd("GETSET").arg(key).arg(value).query_async(&mut self.conn).await
    }

    /// 递增
    pub async fn incr(&mut self, key: &str) -> RedisResult<i64> {
        redis::cmd("INCR").arg(key).query_async(&mut self.conn).await
    }

    /// 递增指定值
    pub async fn incrby(&mut self, key: &str, increment: i64) -> RedisResult<i64> {
        redis::cmd("INCRBY").arg(key).arg(increment).query_async(&mut self.conn).await
    }

    /// 递增浮点数
    pub async fn incrbyfloat(&mut self, key: &str, increment: f64) -> RedisResult<f64> {
        redis::cmd("INCRBYFLOAT").arg(key).arg(increment).query_async(&mut self.conn).await
    }

    /// 递减
    pub async fn decr(&mut self, key: &str) -> RedisResult<i64> {
        redis::cmd("DECR").arg(key).query_async(&mut self.conn).await
    }

    /// 递减指定值
    pub async fn decrby(&mut self, key: &str, decrement: i64) -> RedisResult<i64> {
        redis::cmd("DECRBY").arg(key).arg(decrement).query_async(&mut self.conn).await
    }

    /// 追加字符串
    pub async fn append(&mut self, key: &str, value: &str) -> RedisResult<usize> {
        redis::cmd("APPEND").arg(key).arg(value).query_async(&mut self.conn).await
    }

    /// 获取字符串长度
    pub async fn strlen(&mut self, key: &str) -> RedisResult<usize> {
        redis::cmd("STRLEN").arg(key).query_async(&mut self.conn).await
    }

    // ==================== Hash 操作 ====================

    /// 设置哈希字段值
    pub async fn hset(&mut self, key: &str, field: &str, value: &str) -> RedisResult<bool> {
        redis::cmd("HSET").arg(key).arg(field).arg(value).query_async(&mut self.conn).await
    }

    /// 设置多个哈希字段值
    pub async fn hmset(&mut self, key: &str, pairs: &[(&str, &str)]) -> RedisResult<()> {
        let mut cmd = redis::cmd("HMSET");
        cmd.arg(key);
        for (field, value) in pairs {
            cmd.arg(field).arg(value);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 获取哈希字段值
    pub async fn hget(&mut self, key: &str, field: &str) -> RedisResult<Option<String>> {
        redis::cmd("HGET").arg(key).arg(field).query_async(&mut self.conn).await
    }

    /// 获取多个哈希字段值
    pub async fn hmget(&mut self, key: &str, fields: &[&str]) -> RedisResult<Vec<Option<String>>> {
        let mut cmd = redis::cmd("HMGET");
        cmd.arg(key);
        for field in fields {
            cmd.arg(field);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 获取所有哈希字段和值
    pub async fn hgetall(&mut self, key: &str) -> RedisResult<Vec<(String, String)>> {
        redis::cmd("HGETALL").arg(key).query_async(&mut self.conn).await
    }

    /// 获取所有哈希字段
    pub async fn hkeys(&mut self, key: &str) -> RedisResult<Vec<String>> {
        redis::cmd("HKEYS").arg(key).query_async(&mut self.conn).await
    }

    /// 获取所有哈希值
    pub async fn hvals(&mut self, key: &str) -> RedisResult<Vec<String>> {
        redis::cmd("HVALS").arg(key).query_async(&mut self.conn).await
    }

    /// 检查哈希字段是否存在
    pub async fn hexists(&mut self, key: &str, field: &str) -> RedisResult<bool> {
        redis::cmd("HEXISTS").arg(key).arg(field).query_async(&mut self.conn).await
    }

    /// 删除哈希字段
    pub async fn hdel(&mut self, key: &str, fields: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(key);
        for field in fields {
            cmd.arg(field);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 获取哈希字段数量
    pub async fn hlen(&mut self, key: &str) -> RedisResult<usize> {
        redis::cmd("HLEN").arg(key).query_async(&mut self.conn).await
    }

    /// 哈希字段递增
    pub async fn hincrby(&mut self, key: &str, field: &str, increment: i64) -> RedisResult<i64> {
        redis::cmd("HINCRBY").arg(key).arg(field).arg(increment).query_async(&mut self.conn).await
    }

    /// 哈希字段递增浮点数
    pub async fn hincrbyfloat(&mut self, key: &str, field: &str, increment: f64) -> RedisResult<f64> {
        redis::cmd("HINCRBYFLOAT").arg(key).arg(field).arg(increment).query_async(&mut self.conn).await
    }

    // ==================== Set 操作 ====================

    /// 添加集合成员
    pub async fn sadd(&mut self, key: &str, members: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("SADD");
        cmd.arg(key);
        for member in members {
            cmd.arg(member);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 移除集合成员
    pub async fn srem(&mut self, key: &str, members: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("SREM");
        cmd.arg(key);
        for member in members {
            cmd.arg(member);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 获取集合所有成员
    pub async fn smembers(&mut self, key: &str) -> RedisResult<HashSet<String>> {
        redis::cmd("SMEMBERS").arg(key).query_async(&mut self.conn).await
    }

    /// 检查成员是否在集合中
    pub async fn sismember(&mut self, key: &str, member: &str) -> RedisResult<bool> {
        redis::cmd("SISMEMBER").arg(key).arg(member).query_async(&mut self.conn).await
    }

    /// 获取集合成员数量
    pub async fn scard(&mut self, key: &str) -> RedisResult<usize> {
        redis::cmd("SCARD").arg(key).query_async(&mut self.conn).await
    }

    /// 随机获取集合成员
    pub async fn srandmember(&mut self, key: &str) -> RedisResult<Option<String>> {
        redis::cmd("SRANDMEMBER").arg(key).query_async(&mut self.conn).await
    }

    /// 随机获取多个集合成员
    pub async fn srandmember_count(&mut self, key: &str, count: i64) -> RedisResult<Vec<String>> {
        redis::cmd("SRANDMEMBER").arg(key).arg(count).query_async(&mut self.conn).await
    }

    /// 随机移除并返回集合成员
    pub async fn spop(&mut self, key: &str) -> RedisResult<Option<String>> {
        redis::cmd("SPOP").arg(key).query_async(&mut self.conn).await
    }

    /// 随机移除并返回多个集合成员
    pub async fn spop_count(&mut self, key: &str, count: usize) -> RedisResult<Vec<String>> {
        redis::cmd("SPOP").arg(key).arg(count).query_async(&mut self.conn).await
    }

    /// 移动成员到另一个集合
    pub async fn smove(&mut self, source: &str, destination: &str, member: &str) -> RedisResult<bool> {
        redis::cmd("SMOVE").arg(source).arg(destination).arg(member).query_async(&mut self.conn).await
    }

    /// 计算多个集合的交集
    pub async fn sinter(&mut self, keys: &[&str]) -> RedisResult<HashSet<String>> {
        let mut cmd = redis::cmd("SINTER");
        for key in keys {
            cmd.arg(key);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 计算多个集合的并集
    pub async fn sunion(&mut self, keys: &[&str]) -> RedisResult<HashSet<String>> {
        let mut cmd = redis::cmd("SUNION");
        for key in keys {
            cmd.arg(key);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 计算多个集合的差集
    pub async fn sdiff(&mut self, keys: &[&str]) -> RedisResult<HashSet<String>> {
        let mut cmd = redis::cmd("SDIFF");
        for key in keys {
            cmd.arg(key);
        }
        cmd.query_async(&mut self.conn).await
    }

    // ==================== ZSet (Sorted Set) 操作 ====================

    /// 添加有序集合成员
    pub async fn zadd(&mut self, key: &str, score: f64, member: &str) -> RedisResult<bool> {
        redis::cmd("ZADD").arg(key).arg(score).arg(member).query_async(&mut self.conn).await
    }

    /// 批量添加有序集合成员
    pub async fn zadd_multiple(&mut self, key: &str, pairs: &[(f64, &str)]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("ZADD");
        cmd.arg(key);
        for (score, member) in pairs {
            cmd.arg(*score).arg(*member);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 获取有序集合成员分数
    pub async fn zscore(&mut self, key: &str, member: &str) -> RedisResult<Option<f64>> {
        redis::cmd("ZSCORE").arg(key).arg(member).query_async(&mut self.conn).await
    }

    /// 获取有序集合成员数量
    pub async fn zcard(&mut self, key: &str) -> RedisResult<usize> {
        redis::cmd("ZCARD").arg(key).query_async(&mut self.conn).await
    }

    /// 计算指定分数范围内的成员数量
    pub async fn zcount(&mut self, key: &str, min: f64, max: f64) -> RedisResult<usize> {
        redis::cmd("ZCOUNT").arg(key).arg(min).arg(max).query_async(&mut self.conn).await
    }

    /// 递增有序集合成员分数
    pub async fn zincrby(&mut self, key: &str, increment: f64, member: &str) -> RedisResult<f64> {
        redis::cmd("ZINCRBY").arg(key).arg(increment).arg(member).query_async(&mut self.conn).await
    }

    /// 获取有序集合排名（从低到高，0开始）
    pub async fn zrank(&mut self, key: &str, member: &str) -> RedisResult<Option<usize>> {
        redis::cmd("ZRANK").arg(key).arg(member).query_async(&mut self.conn).await
    }

    /// 获取有序集合排名（从高到低，0开始）
    pub async fn zrevrank(&mut self, key: &str, member: &str) -> RedisResult<Option<usize>> {
        redis::cmd("ZREVRANK").arg(key).arg(member).query_async(&mut self.conn).await
    }

    /// 按排名范围获取有序集合成员（从低到高）
    pub async fn zrange(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<String>> {
        redis::cmd("ZRANGE").arg(key).arg(start).arg(stop).query_async(&mut self.conn).await
    }

    /// 按排名范围获取有序集合成员和分数（从低到高）
    pub async fn zrange_withscores(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<(String, f64)>> {
        redis::cmd("ZRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut self.conn)
            .await
    }

    /// 按排名范围获取有序集合成员（从高到低）
    pub async fn zrevrange(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<String>> {
        redis::cmd("ZREVRANGE").arg(key).arg(start).arg(stop).query_async(&mut self.conn).await
    }

    /// 按排名范围获取有序集合成员和分数（从高到低）
    pub async fn zrevrange_withscores(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<(String, f64)>> {
        redis::cmd("ZREVRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut self.conn)
            .await
    }

    /// 按分数范围获取有序集合成员
    pub async fn zrangebyscore(&mut self, key: &str, min: f64, max: f64) -> RedisResult<Vec<String>> {
        redis::cmd("ZRANGEBYSCORE").arg(key).arg(min).arg(max).query_async(&mut self.conn).await
    }

    /// 按分数范围获取有序集合成员和分数
    pub async fn zrangebyscore_withscores(&mut self, key: &str, min: f64, max: f64) -> RedisResult<Vec<(String, f64)>> {
        redis::cmd("ZRANGEBYSCORE")
            .arg(key)
            .arg(min)
            .arg(max)
            .arg("WITHSCORES")
            .query_async(&mut self.conn)
            .await
    }

    /// 按分数范围获取有序集合成员（从高到低）
    pub async fn zrevrangebyscore(&mut self, key: &str, max: f64, min: f64) -> RedisResult<Vec<String>> {
        redis::cmd("ZREVRANGEBYSCORE").arg(key).arg(max).arg(min).query_async(&mut self.conn).await
    }

    /// 按分数范围获取有序集合成员和分数（从高到低）
    pub async fn zrevrangebyscore_withscores(&mut self, key: &str, max: f64, min: f64) -> RedisResult<Vec<(String, f64)>> {
        redis::cmd("ZREVRANGEBYSCORE")
            .arg(key)
            .arg(max)
            .arg(min)
            .arg("WITHSCORES")
            .query_async(&mut self.conn)
            .await
    }

    /// 移除有序集合成员
    pub async fn zrem(&mut self, key: &str, members: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("ZREM");
        cmd.arg(key);
        for member in members {
            cmd.arg(member);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 按排名范围移除有序集合成员
    pub async fn zremrangebyrank(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<usize> {
        redis::cmd("ZREMRANGEBYRANK").arg(key).arg(start).arg(stop).query_async(&mut self.conn).await
    }

    /// 按分数范围移除有序集合成员
    pub async fn zremrangebyscore(&mut self, key: &str, min: f64, max: f64) -> RedisResult<usize> {
        redis::cmd("ZREMRANGEBYSCORE").arg(key).arg(min).arg(max).query_async(&mut self.conn).await
    }

    // ==================== List 操作 ====================

    /// 从列表左侧推入元素
    pub async fn lpush(&mut self, key: &str, values: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("LPUSH");
        cmd.arg(key);
        for value in values {
            cmd.arg(value);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 从列表右侧推入元素
    pub async fn rpush(&mut self, key: &str, values: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("RPUSH");
        cmd.arg(key);
        for value in values {
            cmd.arg(value);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 从列表左侧弹出元素
    pub async fn lpop(&mut self, key: &str) -> RedisResult<Option<String>> {
        redis::cmd("LPOP").arg(key).query_async(&mut self.conn).await
    }

    /// 从列表右侧弹出元素
    pub async fn rpop(&mut self, key: &str) -> RedisResult<Option<String>> {
        redis::cmd("RPOP").arg(key).query_async(&mut self.conn).await
    }

    /// 从列表左侧弹出元素（阻塞，超时时间：秒）
    pub async fn blpop(&mut self, keys: &[&str], timeout: usize) -> RedisResult<Option<(String, String)>> {
        let mut cmd = redis::cmd("BLPOP");
        for key in keys {
            cmd.arg(key);
        }
        cmd.arg(timeout).query_async(&mut self.conn).await
    }

    /// 从列表右侧弹出元素（阻塞，超时时间：秒）
    pub async fn brpop(&mut self, keys: &[&str], timeout: usize) -> RedisResult<Option<(String, String)>> {
        let mut cmd = redis::cmd("BRPOP");
        for key in keys {
            cmd.arg(key);
        }
        cmd.arg(timeout).query_async(&mut self.conn).await
    }

    /// 获取列表长度
    pub async fn llen(&mut self, key: &str) -> RedisResult<usize> {
        redis::cmd("LLEN").arg(key).query_async(&mut self.conn).await
    }

    /// 获取列表指定范围的元素
    pub async fn lrange(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<String>> {
        redis::cmd("LRANGE").arg(key).arg(start).arg(stop).query_async(&mut self.conn).await
    }

    /// 获取列表指定索引的元素
    pub async fn lindex(&mut self, key: &str, index: isize) -> RedisResult<Option<String>> {
        redis::cmd("LINDEX").arg(key).arg(index).query_async(&mut self.conn).await
    }

    /// 设置列表指定索引的元素
    pub async fn lset(&mut self, key: &str, index: isize, value: &str) -> RedisResult<()> {
        redis::cmd("LSET").arg(key).arg(index).arg(value).query_async(&mut self.conn).await
    }

    /// 在指定元素前插入新元素
    pub async fn linsert_before(&mut self, key: &str, pivot: &str, value: &str) -> RedisResult<isize> {
        redis::cmd("LINSERT").arg(key).arg("BEFORE").arg(pivot).arg(value).query_async(&mut self.conn).await
    }

    /// 在指定元素后插入新元素
    pub async fn linsert_after(&mut self, key: &str, pivot: &str, value: &str) -> RedisResult<isize> {
        redis::cmd("LINSERT").arg(key).arg("AFTER").arg(pivot).arg(value).query_async(&mut self.conn).await
    }

    /// 移除列表中指定值的元素
    pub async fn lrem(&mut self, key: &str, count: isize, value: &str) -> RedisResult<usize> {
        redis::cmd("LREM").arg(key).arg(count).arg(value).query_async(&mut self.conn).await
    }

    /// 修剪列表，只保留指定范围内的元素
    pub async fn ltrim(&mut self, key: &str, start: isize, stop: isize) -> RedisResult<()> {
        redis::cmd("LTRIM").arg(key).arg(start).arg(stop).query_async(&mut self.conn).await
    }

    // ==================== 通用操作 ====================

    /// 删除键
    pub async fn del(&mut self, keys: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("DEL");
        for key in keys {
            cmd.arg(key);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 检查键是否存在
    pub async fn exists(&mut self, keys: &[&str]) -> RedisResult<usize> {
        let mut cmd = redis::cmd("EXISTS");
        for key in keys {
            cmd.arg(key);
        }
        cmd.query_async(&mut self.conn).await
    }

    /// 设置键的过期时间（秒）
    pub async fn expire(&mut self, key: &str, seconds: usize) -> RedisResult<bool> {
        redis::cmd("EXPIRE").arg(key).arg(seconds).query_async(&mut self.conn).await
    }

    /// 设置键的过期时间（毫秒）
    pub async fn pexpire(&mut self, key: &str, milliseconds: usize) -> RedisResult<bool> {
        redis::cmd("PEXPIRE").arg(key).arg(milliseconds).query_async(&mut self.conn).await
    }

    /// 设置键在指定时间戳过期（秒）
    pub async fn expireat(&mut self, key: &str, timestamp: i64) -> RedisResult<bool> {
        redis::cmd("EXPIREAT").arg(key).arg(timestamp).query_async(&mut self.conn).await
    }

    /// 设置键在指定时间戳过期（毫秒）
    pub async fn pexpireat(&mut self, key: &str, timestamp: i64) -> RedisResult<bool> {
        redis::cmd("PEXPIREAT").arg(key).arg(timestamp).query_async(&mut self.conn).await
    }

    /// 获取键的剩余过期时间（秒）
    pub async fn ttl(&mut self, key: &str) -> RedisResult<isize> {
        redis::cmd("TTL").arg(key).query_async(&mut self.conn).await
    }

    /// 获取键的剩余过期时间（毫秒）
    pub async fn pttl(&mut self, key: &str) -> RedisResult<isize> {
        redis::cmd("PTTL").arg(key).query_async(&mut self.conn).await
    }

    /// 移除键的过期时间
    pub async fn persist(&mut self, key: &str) -> RedisResult<bool> {
        redis::cmd("PERSIST").arg(key).query_async(&mut self.conn).await
    }

    /// 获取键的类型
    pub async fn type_(&mut self, key: &str) -> RedisResult<String> {
        redis::cmd("TYPE").arg(key).query_async(&mut self.conn).await
    }

    /// 重命名键
    pub async fn rename(&mut self, key: &str, new_key: &str) -> RedisResult<()> {
        redis::cmd("RENAME").arg(key).arg(new_key).query_async(&mut self.conn).await
    }

    /// 如果新键不存在则重命名
    pub async fn renamenx(&mut self, key: &str, new_key: &str) -> RedisResult<bool> {
        redis::cmd("RENAMENX").arg(key).arg(new_key).query_async(&mut self.conn).await
    }
}
