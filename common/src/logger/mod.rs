// 日志模块
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use chrono::Local;

/// 初始化日志系统
/// 
/// 支持通过环境变量 RUST_LOG 配置日志级别
/// 例如: RUST_LOG=debug 或 RUST_LOG=info
pub fn init_logger() {
    let mut builder = Builder::new();
    
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter_level(get_log_level_from_env())
        .init();
    
    log::info!("✅ 日志系统初始化完成");
}

/// 从环境变量获取日志级别
fn get_log_level_from_env() -> LevelFilter {
    match std::env::var("RUST_LOG") {
        Ok(level) => match level.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info,
        },
        Err(_) => LevelFilter::Info,
    }
}

/// 初始化日志系统（带自定义级别）
pub fn init_logger_with_level(level: LevelFilter) {
    let mut builder = Builder::new();
    
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter_level(level)
        .init();
    
    log::info!("✅ 日志系统初始化完成 (级别: {:?})", level);
}