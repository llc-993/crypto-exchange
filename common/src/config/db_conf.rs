use rbatis::RBatis;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};
use once_cell::sync::OnceCell;
use rbdc_mysql::driver::MysqlDriver;

static DB: OnceCell<RBatis> = OnceCell::new();

/// MySQL 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// 数据库连接 URL
    pub url: String,
    /// 连接池最大连接数
    pub max_connections: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: "mysql://root:password@localhost:3306/crypto_exchange".to_string(),
            max_connections: 10,
        }
    }
}

impl DbConfig {
    /// 创建新的数据库配置
    pub fn new(url: String, max_connections: u64) -> Self {
        Self {
            url,
            max_connections,
        }
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:password@localhost:3306/crypto_exchange".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }

    /// 构建带连接池参数的数据库 URL
    pub fn build_url_with_pool(&self) -> String {
        if self.url.contains('?') {
            format!("{}&max_connections={}", self.url, self.max_connections)
        } else {
            format!("{}?max_connections={}", self.url, self.max_connections)
        }
    }
}

/// 初始化数据库连接
///
/// # Arguments
/// * `config` - 数据库配置
///
/// # Errors
/// 如果数据库已初始化或连接失败则返回错误
pub async fn init_db(config: &DbConfig) -> AppResult<()> {
    let rb = RBatis::new();
    // 初始化数据库连接
    rb.init(MysqlDriver {}, &config.build_url_with_pool())
        .map_err(|e| AppError::database_error(e.to_string()))?;

    // 设置全局实例
    DB.set(rb)
        .map_err(|_| AppError::database_error("Database already initialized".to_string()))?;

    log::info!("✅ 数据库连接初始化成功");
    Ok(())
}

/// 获取数据库实例
///
/// # Panics
/// 如果数据库未初始化则会 panic
pub fn get_db() -> &'static RBatis {
    DB.get().expect("数据库未初始化，请先调用 init_db()")
}

/// 测试数据库连接
pub async fn test_connection() -> AppResult<bool> {
    let rb = get_db();
    let result = rb.query("SELECT 1", vec![]).await;
    match result {
        Ok(_) => {
            log::info!("✅ 数据库连接测试成功");
            Ok(true)
        }
        Err(e) => {
            log::error!("❌ 数据库连接测试失败: {}", e);
            Err(AppError::database_error(e.to_string()))
        }
    }
}

/// 获取连接池状态（如果可用）
pub fn get_pool_status() -> String {
    let rb = get_db();
    format!("数据库连接池状态: {:?}", rb)
}
