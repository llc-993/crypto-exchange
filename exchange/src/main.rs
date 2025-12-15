mod consumer;
mod config;
mod common;

use ::common::AppConfig;

#[tokio::main]
async fn main() {
    // 嵌入配置文件（编译时加载）
    const DEFAULT_CONFIG: &str = include_str!("../config.toml");
    const PROD_CONFIG: &str = include_str!("../config.production.toml");

    let app_config = AppConfig::from_file_or_embedded(
        "exchange/config",
        DEFAULT_CONFIG,
        Some(PROD_CONFIG)
    )
    .or_else(|_| AppConfig::from_env())
    .expect("配置加载失败"); 

    // 初始化日志（使用配置的日志级别）
    std::env::set_var("RUST_LOG", &app_config.log.level);
    ::common::init_logger();
    
    log::info!("启动交易所核心服务...");

    log::info!("配置加载成功 - 数据库: {}", app_config.database.url);
    
    // 初始化数据库连接
    let db_config = ::common::DbConfig::new(
        app_config.database.url.clone(),
        app_config.database.max_connections as u64,
    );
    ::common::init_db(&db_config)
        .await
        .expect("数据库连接池初始化失败");
    
    // 测试数据库连接
    if let Err(e) = ::common::test_db_connection().await {
        log::error!("数据库连接测试失败: {}", e);
    }
    
    // 加载交易对配置到缓存
    common::coin_cache::init_all_caches();

    // 初始化 Redis 连接
    let redis_config = ::common::RedisConfig::from_url(
        app_config.redis.url.clone(),
        app_config.redis.pool_size,
    );
    let mut redis_conn = ::common::create_async_connection_from_config(&redis_config)
        .await
        .expect("Redis初始化失败");
    
    // 测试 Redis 连接
    if let Err(e) = ::common::test_redis_connection(&mut redis_conn).await {
        log::error!("Redis连接测试失败: {}", e);
    }

    // 初始化 Ticker 缓存（从 Redis 加载历史数据）
    common::ticker_cache::init_ticker_caches_from_redis().await;
    
    // 初始化 MarkPrice 缓存（从 Redis 加载历史数据）
    common::mark_price_cache::init_mark_price_caches_from_redis().await;

    // 初始化 Pulsar Client
    if app_config.pulsar.enabled {
        log::info!("正在初始化 Pulsar Client: {}", app_config.pulsar.url);
        if let Err(e) = ::common::PulsarClient::init_global(&app_config.pulsar.url).await {
            log::error!("Pulsar 初始化失败: {}", e);
        } else {
            // 启动消费者
            consumer::start_ticker_consumer().await;
        }
    } else {
        log::warn!("Pulsar 未启用，跳过初始化");
    }
    // 初始化 Disruptor 消息处理引擎
    config::disruptor_config::init(&app_config.disruptor);
    
    log::info!("Exchange服务启动在: {}:{}", app_config.server.host, app_config.server.port);
    
    // 保持服务运行
    log::info!("服务正在运行中，按 Ctrl+C 退出...");
    
    // 使用 tokio::signal 等待退出信号
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c signal");
    
    log::info!("收到退出信号，正在关闭服务...");
    
    // 保存缓存数据到 Redis
    log::info!("正在保存缓存数据到 Redis...");
    common::ticker_cache::save_all_tickers_to_redis().await;
    common::mark_price_cache::save_all_mark_prices_to_redis().await;
}