// 公共模块
// 提供数据库、Redis、日志、错误处理等通用功能

rust_i18n::i18n!("locales");


pub mod config;
pub mod database;
pub mod redis;
pub mod error;
pub mod logger;
pub mod enums;
pub mod pulsar;
pub mod mongodb;
pub mod models;
pub mod response;
pub mod middleware;
pub mod constants;
pub mod utils;
pub mod services;
// 重新导出常用类型和函数
pub use error::{AppError, AppResult};
pub use config::{DbConfig, RedisConfig, AppConfig, MongoDBConfig}; 
pub use logger::{init_logger, init_logger_with_level};
pub use enums::{KlineInterval, KlineIntervalConfig};

// 数据库相关
pub use database::{init_db, get_db, test_connection as test_db_connection};

// Redis 相关
pub use redis::{
    create_client as create_redis_client,
    create_async_connection,
    create_async_connection_from_config,
    test_connection as test_redis_connection,
    cache,
};

// Pulsar 相关
pub use pulsar::{PulsarClient, Event};

// MongoDB 相关
pub use mongodb::MongoDBClient;

// 数据模型
pub use models::{
    UnifiedTicker, 
    UnifiedMarkPrice,
    TickerConverter, 
    MarkPriceConverter,
    MarketType
};

/// 初始化公共模块
/// 
/// 这个函数可以用来初始化日志系统
pub fn init() {
    logger::init_logger();
    log::info!("✅ 公共模块初始化完成");
}

/// 初始化公共模块（带自定义日志级别）
pub fn init_with_log_level(level: log::LevelFilter) {
    logger::init_logger_with_level(level);
    log::info!("✅ 公共模块初始化完成");
}
