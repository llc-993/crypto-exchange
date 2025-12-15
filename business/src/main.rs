
use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use rbatis::RBatis;
use sa_token_plugin_actix_web::{RedisStorage, SaTokenConfig, SaTokenState};
use common::AppConfig;
use common::constants::{SA_TOKEN_AUTH_HEADER_NAME, SA_TOKEN_KEY_PREFIX};
use common::middleware::error_handler;
use middleware::i18n::I18n;
use common::middleware::sa_token::sa_token_middleware::SaTokenMiddleware;
use common::middleware::sa_token::auth_checker::DefaultAuthChecker;
use common::services::config_service::ConfigService;
use common::services::ip_service::IpService;
use common::services::upload::UploadServiceSupport;
use common::utils::redis_util::RedisUtil;
use common::services::email::EmailServiceSupport;
use common::services::emqx_service::EmqxService;
use common::services::sms::SmsServiceSupport;
use common::mq::message_queue::MessageQueue;
use crate::service::agent_relation_service::AgentRelationService;

mod handle;
mod service;
mod middleware;
mod config;
mod state;
mod subscribers;

//#[tokio::main]
#[actix_web::main]
async fn main()  -> std::io::Result<()>{
    // 嵌入配置文件（编译时加载）
    const DEFAULT_CONFIG: &str = include_str!("../config.toml");
    const PROD_CONFIG: &str = include_str!("../config.production.toml");

    let config = AppConfig::from_file_or_embedded(
        "business/config",
        DEFAULT_CONFIG,
        Some(PROD_CONFIG)
    )
    .or_else(|_| AppConfig::from_env())
    .expect("配置加载失败");

    // 初始化日志（使用配置的日志级别）
    std::env::set_var("RUST_LOG", &config.log.level);
    common::init_logger();
    
    log::info!("启动用户API服务...");
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
    
    log::info!("Business服务启动在: {}:{}", config.server.host, config.server.port);
    
    // 保持服务运行
    /*log::info!("服务正在运行中，按 Ctrl+C 退出...");
    
    // 使用 tokio::signal 等待退出信号
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c signal");
    
    log::info!("收到退出信号，正在关闭服务...");*/
    
    //  初始化 sa-token (使用 Redis 存储)
    // 初始化 Redis 存储
    let redis_storage = RedisStorage::new(&config.redis.url, SA_TOKEN_KEY_PREFIX)
        .await
        .map_err(|e| {
            log::error!("Redis 连接失败: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;

    // 初始化 Sa-Token Manager
    let sa_token_manager = SaTokenConfig::builder()
        .storage(Arc::new(redis_storage))
        .token_name(SA_TOKEN_AUTH_HEADER_NAME)
        .timeout(86400) // 24 小时
        .build();

    let sa_token_middleware = SaTokenMiddleware::builder()
        .state(SaTokenState { manager: Arc::new(sa_token_manager.clone()) })
        .auth_checker(Arc::new(
            DefaultAuthChecker::builder()
                .add_match("/api/**")
                .add_exclude("/api/common/**")
                .add_exclude("/api/auth/**")
                .add_exclude("/api/message/list")
                .add_exclude("/api/prod/**")
                .build()
        ))
        .build();

    // 初始化 RBatis
    let rb = RBatis::new();
    rb.link(rbdc_mysql::MysqlDriver {}, &config.database.url)
        .await
        .map_err(|e| {
            log::error!("数据库连接失败: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
    log::info!("✅ 数据库连接成功");
    let rb = Arc::new(rb);

    // 初始化 Redis 连接池
    log::info!("⚡ 初始化 Redis 连接池...");
    let redis_util = RedisUtil::from_url(config.redis.url)
        .expect("初始化 Redis连接池失败");
    let redis_util = Arc::new(redis_util); // Wrap in Arc
    log::info!("📦 Redis 连接池已就绪");

    let config_service = ConfigService::new(rb.clone(), redis_util.clone());
    let ip_service = IpService::new(redis_util.clone());
    let upload_service_support = UploadServiceSupport::new(rb.clone(), redis_util.clone())
        .await;

    let email_service = EmailServiceSupport::new(rb.clone(), redis_util.clone())
        .await;

    let sms_service = SmsServiceSupport::new(rb.clone(), redis_util.clone())
        .await;

    let config_service_arc = Arc::new(config_service);
    let emqx_service = EmqxService::new(config_service_arc.clone());
    // redis-mq
    let mq = MessageQueue::new(redis_util.clone());

    let agent_relation_service = AgentRelationService::new(rb.clone());
    // 组装工程依赖
    let state = state::AppState {
        rb,
        redis: redis_util,
        config_service: config_service_arc,
        ip_service: Arc::new(ip_service),
        upload_service: Arc::new(upload_service_support),
        email_service: Arc::new(email_service),
        emqx_service: Arc::new(emqx_service),
        sms_service: Arc::new(sms_service),
        mq: Arc::new(mq),
        agent_relation_service: Arc::new(agent_relation_service),
    };
    let state_data = web::Data::new(state.clone());

    // 注册消息队列订阅者
    subscribers::init_subscribers(state_data.clone()).await;

    let addr = format!("{}:{}", config.server.host, config.server.port);
    log::info!("🚀 启动 Actix Web 服务器...");
    HttpServer::new(move || {
        App::new()
            // 全局中间件配置
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            // 注册 i18n 中间件（在 Sa-Token 之前，确保语言先设置）
            .wrap(I18n)
            // Sa-Token 中间件
            .wrap(sa_token_middleware.clone())
            // 注册 JSON 和 Query 错误处理器
            .app_data(error_handler::json_config())
            .app_data(error_handler::query_config())
            // 注册全局数据
            .app_data(state_data.clone()) // Inject AppState
            .service(handle::common::test)
            .service(handle::common::test_query)
            .service(handle::common::test_body)
            .service(handle::common::query_ip_address)
            .service(handle::common::config)
            .service(handle::common::upload_image)
            .service(handle::user::login)
    }).bind(&addr)?
        .run()
        .await
}