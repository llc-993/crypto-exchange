#[tokio::main]
async fn main() {
    // 初始化日志
    common::logger::init_logger();
    
    log::info!("启动比特币区块链服务...");
}