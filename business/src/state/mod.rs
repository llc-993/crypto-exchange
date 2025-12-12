use std::sync::Arc;
use rbatis::RBatis;
use common::utils::redis_util::RedisUtil;

#[derive(Clone)]
pub struct AppState {
    pub rb: Arc<RBatis>,
    pub redis: Arc<RedisUtil>,
}