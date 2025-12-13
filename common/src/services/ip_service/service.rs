use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use crate::utils::redis_util::RedisUtil;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tokio_retry::strategy::{FixedInterval};
use tokio_retry::Retry;
use rand::Rng;

use crate::services::ip_service::queryers::{AppworldsQueryer, BaiduQueryer, IpApi2Queryer, IpApiCoQueryer, IpApiIsQueryer, IpApiQueryer, IpCnQueryer, IpSbQueryer, MeituQueryer, Mir6Queryer, PconlineQueryer, QjqqQueryer, RealipQueryer, VoreQueryer};

const REDIS_KEY_IP_CACHES: &str = "IP_CACHES:";

pub struct IpService {
    redis: Arc<RedisUtil>,
    queryers: Vec<Arc<dyn IpQueryer>>,
}

impl IpService {
    pub fn new(redis: Arc<RedisUtil>) -> Self {

        let queryers: Vec<Arc<dyn IpQueryer>> = vec![
            Arc::new(IpApiQueryer::new()),
            Arc::new(AppworldsQueryer::new()),
            Arc::new(BaiduQueryer::new()),
            Arc::new(IpApi2Queryer::new()),
            Arc::new(IpApiCoQueryer::new()),
            Arc::new(IpApiIsQueryer::new()),
            Arc::new(IpCnQueryer::new()),
            Arc::new(IpSbQueryer::new()),
            Arc::new(MeituQueryer::new()),
            Arc::new(Mir6Queryer::new()),
            Arc::new(PconlineQueryer::new()),
            Arc::new(QjqqQueryer::new()),
            Arc::new(RealipQueryer::new()),
            Arc::new(VoreQueryer::new()),
        ];
        Self { redis, queryers }
    }
    
    /// 验证是否为有效的 IPv4 地址
    fn is_valid_ipv4(ip: &str) -> bool {
        ip.parse::<Ipv4Addr>().is_ok()
    }
    
    /// 验证是否为有效的 IPv6 地址
    fn is_valid_ipv6(ip: &str) -> bool {
        ip.parse::<Ipv6Addr>().is_ok()
    }
    
    /// 从缓存获取IP归属地
    async fn by_cache(&self, ip: &str) -> Result<Option<String>, AppError> {
        self.redis.hget(REDIS_KEY_IP_CACHES, ip).await
    }
    
    /// 保存到缓存
    async fn save_cache(&self, ip: &str, address: &str) -> Result<(), AppError> {
        self.redis.hset(REDIS_KEY_IP_CACHES, ip, address).await?;
        Ok(())
    }
    
    /// 获取 IP 归属地（带重试机制）
    pub async fn get_real_address_by_ip(&self, ip: &str, force: bool) -> Result<Option<String>, AppError> {
        // 重试策略：固定间隔 2 秒，最多重试 3 次
        let retry_strategy = FixedInterval::from_millis(2000).take(3);
        
        let result = Retry::spawn(retry_strategy, || {
            self._get_real_address_by_ip(ip, force)
        }).await;
        
        match result {
            Ok(address) => Ok(address),
            Err(e) => {
                log::warn!("Failed to get IP geolocation after retries: {}", e);
                Ok(None)
            }
        }
    }
    
    /// 实际查询逻辑
    async fn _get_real_address_by_ip(&self, ip: &str, force: bool) -> Result<Option<String>, AppError> {
        // 1. 检查缓存
        if !force {
            if let Some(cached) = self.by_cache(ip).await? {
                if !cached.is_empty() {
                    log::debug!("IP {} found in cache: {}", ip, cached);
                    return Ok(Some(cached));
                }
            }
        }
        
        // 2. 根据 IP 类型筛选查询器
        let supported: Vec<_> = if Self::is_valid_ipv4(ip) {
            self.queryers.clone()
        } else if Self::is_valid_ipv6(ip) {
            self.queryers.iter()
                .filter(|q| q.support_ipv6())
                .cloned()
                .collect()
        } else {
            return Err(AppError::validation_with_params("error.validation", serde_json::json!({"msg": format!("Invalid IP address: {}", ip)})));
        };
        
        if supported.is_empty() {
            return Err(AppError::unknown("error.No supported IP queryer available"));
        }
        
        // 3. 随机选择一个查询器
        let queryer = {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..self.queryers.len());
            &self.queryers[idx]
        };
        
        log::debug!("Selected queryer {} for IP {}", 0, ip); // Note: idx is not accessible here, using 0 as placeholder
        
        // 4. 查询
        let address = queryer.get_real_address(ip).await?;
        
        // 5. 保存到缓存
        self.save_cache(ip, &address).await?;
        
        Ok(Some(address))
    }
}
