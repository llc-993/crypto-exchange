use serde::{Deserialize, Serialize};

/// Redis 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis 主机地址
    #[serde(default = "default_host")]
    pub host: String,
    /// Redis 端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// Redis 密码（可选）
    #[serde(default)]
    pub password: Option<String>,
    /// 数据库编号
    #[serde(default)]
    pub database: u8,
    /// 连接池大小
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    6379
}

fn default_pool_size() -> u32 {
    10
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            database: 0,
            pool_size: 10,
        }
    }
}

impl RedisConfig {
    /// 创建新的 Redis 配置
    pub fn new(host: String, port: u16, password: Option<String>, database: u8, pool_size: u32) -> Self {
        Self {
            host,
            port,
            password,
            database,
            pool_size,
        }
    }

    /// 从完整 URL 创建配置（兼容旧方式）
    pub fn from_url(url: String, pool_size: u32) -> Self {
        // 简单解析，实际使用时 redis 客户端会处理完整 URL
        Self {
            host: url.clone(),
            port: 0, // URL 模式不使用
            password: None,
            database: 0,
            pool_size,
        }
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        // 优先使用 REDIS_URL，如果不存在则使用独立配置
        if let Ok(url) = std::env::var("REDIS_URL") {
            Self::from_url(url, 
                std::env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10))
        } else {
            Self {
                host: std::env::var("REDIS_HOST")
                    .unwrap_or_else(|_| "localhost".to_string()),
                port: std::env::var("REDIS_PORT")
                    .unwrap_or_else(|_| "6379".to_string())
                    .parse()
                    .unwrap_or(6379),
                password: std::env::var("REDIS_PASSWORD").ok(),
                database: std::env::var("REDIS_DATABASE")
                    .unwrap_or_else(|_| "0".to_string())
                    .parse()
                    .unwrap_or(0),
                pool_size: std::env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            }
        }
    }

    /// 构建 Redis 连接 URL
    /// 
    /// 格式：
    /// - 无密码: redis://host:port/database
    /// - 有密码: redis://:password@host:port/database
    pub fn build_url(&self) -> String {
        // 如果 port 为 0，说明使用的是完整 URL 模式
        if self.port == 0 {
            return self.host.clone();
        }

        // 构建标准格式的 URL
        if let Some(password) = &self.password {
            if !password.is_empty() {
                // 有密码: redis://:password@host:port/database
                format!(
                    "redis://:{}@{}:{}/{}",
                    password, self.host, self.port, self.database
                )
            } else {
                // 密码为空字符串，视为无密码
                format!("redis://{}:{}/{}", self.host, self.port, self.database)
            }
        } else {
            // 无密码: redis://host:port/database
            format!("redis://{}:{}/{}", self.host, self.port, self.database)
        }
    }

    /// 获取连接 URL（别名方法，兼容旧代码）
    pub fn get_url(&self) -> String {
        self.build_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_without_password() {
        let config = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            database: 0,
            pool_size: 10,
        };
        assert_eq!(config.build_url(), "redis://localhost:6379/0");
    }

    #[test]
    fn test_build_url_with_password() {
        let config = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: Some("mypassword".to_string()),
            database: 1,
            pool_size: 10,
        };
        assert_eq!(config.build_url(), "redis://:mypassword@localhost:6379/1");
    }

    #[test]
    fn test_build_url_from_full_url() {
        let config = RedisConfig::from_url("redis://remote:6379/2".to_string(), 20);
        assert_eq!(config.build_url(), "redis://remote:6379/2");
    }
}
