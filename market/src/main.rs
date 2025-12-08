use common::AppConfig;

#[tokio::main]
async fn main() {
    // 嵌入配置文件（编译时加载）
    const DEFAULT_CONFIG: &str = include_str!("../config.toml");
    const PROD_CONFIG: &str = include_str!("../config.production.toml");
    
    let config = AppConfig::from_file_or_embedded(
        "market/config",
        DEFAULT_CONFIG,
        Some(PROD_CONFIG)
    )
    .or_else(|_| AppConfig::from_env())
    .expect("配置加载失败");

    // 初始化日志（使用配置的日志级别）
    std::env::set_var("RUST_LOG", &config.log.level);
    common::init_logger();
    
    log::info!("启动行情服务...");
    log::info!("配置加载成功 - 数据库: {}", config.database.url);
    
    // 初始化数据库连接
    let db_config = common::DbConfig::new(
        config.database.url.clone(),
        config.database.max_connections as u64,
    );
    common::init_db(&db_config)
        .await
        .expect("数据库连接池初始化失败");
    
    // 测试数据库连接
    if let Err(e) = common::test_db_connection().await {
        log::error!("数据库连接测试失败: {}", e);
    }
    
    // 初始化 Redis 连接
    let redis_config = common::RedisConfig::from_url(
        config.redis.url.clone(),
        config.redis.pool_size,
    );
    let mut redis_conn = common::create_async_connection_from_config(&redis_config)
        .await
        .expect("Redis初始化失败");
    
    // 测试 Redis 连接
    if let Err(e) = common::test_redis_connection(&mut redis_conn).await {
        log::error!("Redis连接测试失败: {}", e);
    }
    
    log::info!("Market服务启动在: {}:{}", config.server.host, config.server.port);
    
    // 保持服务运行
    log::info!("服务正在运行中，按 Ctrl+C 退出...");
    
    // 使用 tokio::signal 等待退出信号
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c signal");
    
    log::info!("收到退出信号，正在关闭服务...");
}