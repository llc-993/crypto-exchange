// 配置模块
pub struct BusinessConfig {
    pub server_port: u16,
    pub server_host: String,
}

pub fn load_config() -> BusinessConfig {
    BusinessConfig {
        server_port: 8080,
        server_host: "0.0.0.0".to_string(),
    }
}
