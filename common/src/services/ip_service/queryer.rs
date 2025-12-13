use crate::error::AppError;
use async_trait::async_trait;

/// IP 查询器 Trait
#[async_trait]
pub trait IpQueryer: Send + Sync {
    /// 是否支持 IPv6
    fn support_ipv6(&self) -> bool;
    
    /// 获取 IP 归属地
    async fn get_real_address(&self, ip: &str) -> Result<String, AppError>;
}
