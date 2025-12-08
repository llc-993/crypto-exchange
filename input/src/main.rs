use input::{BinanceSpot, BinanceFutures, BitgetSpot, BitgetFutures, OkxSpot, OkxFutures};
use common::AppConfig;

#[tokio::main]
async fn main() {
    // 嵌入配置文件（编译时加载）
    const DEFAULT_CONFIG: &str = include_str!("../config.toml");
    const PROD_CONFIG: &str = include_str!("../config.production.toml");

    let config = AppConfig::from_file_or_embedded(
        "input/config",
        DEFAULT_CONFIG,
        Some(PROD_CONFIG),
    )
        .or_else(|_| AppConfig::from_env())
        .expect("配置加载失败");

    // 初始化日志（使用配置的日志级别）
    std::env::set_var("RUST_LOG", &config.log.level);
    common::init_logger();

    log::info!("启动外部交易所数据接入服务...");
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

    // 初始化全局 Pulsar 客户端
    if config.pulsar.enabled {
        log::info!("初始化 Pulsar 客户端...");
        if let Err(e) = common::PulsarClient::init_global(&config.pulsar.url).await {
            log::error!("❌ Pulsar 初始化失败: {}", e);
        } else {
            log::info!("✅ Pulsar 客户端连接成功: {}", config.pulsar.url);
        }
    } else {
        log::info!("⏭️ Pulsar 已禁用，跳过初始化");
    }

    // 初始化各交易所模块
    log::info!("初始化 Binance 模块...");
    let binance_spot = BinanceSpot::new();
    if let Err(e) = binance_spot.load_spot_coins().await {
        log::warn!("Binance 现货交易对加载失败: {}", e);
    } else {
        if let Err(e) = binance_spot.start().await {
            log::error!("Binance 现货数据流启动失败: {}", e);
        }
    }

    let binance_futures = BinanceFutures::new();
    if let Err(e) = binance_futures.load_futures_coins().await {
        log::warn!("Binance 永续合约配置加载失败: {}", e);
    } else {
        if let Err(e) = binance_futures.start().await {
            log::error!("Binance 永续合约数据流启动失败: {}", e);
        }
    }
    log::info!("✅ Binance 模块初始化完成");

    log::info!("初始化 Bitget 模块...");
    let bitget_spot = BitgetSpot::new();
    if let Err(e) = bitget_spot.load_spot_coins().await {
        log::warn!("Bitget 现货交易对加载失败: {}", e);
    } else {
        if let Err(e) = bitget_spot.start().await {
            log::error!("bitget 现货启动失败: {}", e);
        }
    }

    let bitget_futures = BitgetFutures::new();
    if let Err(e) = bitget_futures.load_futures_coins().await {
        log::warn!("Bitget 永续合约配置加载失败: {}", e);
    } else {
        if let Err(e) = bitget_futures.start().await {
            log::error!("Bitget 永续合约数据流启动失败: {}", e);
        }
    }
    log::info!("✅ Bitget 模块初始化完成");

    log::info!("初始化 OKX 模块...");
    let okx_spot = OkxSpot::new();
    if let Err(e) = okx_spot.load_spot_coins().await {
        log::warn!("OKX 现货交易对加载失败: {}", e);
    } else {
        if let Err(e) = okx_spot.start().await {
            log::error!("OKX 现货数据流启动失败: {}", e);
        }
    }

    let okx_futures = OkxFutures::new();
    if let Err(e) = okx_futures.load_futures_coins().await {
        log::warn!("OKX 永续合约配置加载失败: {}", e);
    } else {
        if let Err(e) = okx_futures.start().await {
            log::error!("OKX 永续合约数据流启动失败: {}", e);
        }
    }
    log::info!("✅ OKX 模块初始化完成");

    log::info!("所有交易所模块已就绪");
    log::info!("Input服务启动在: {}:{}", config.server.host, config.server.port);

    // 保持服务运行
    log::info!("服务正在运行中，按 Ctrl+C 退出...");

    // 使用 tokio::signal 等待退出信号
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c signal");

    log::info!("收到退出信号，正在关闭服务...");
}