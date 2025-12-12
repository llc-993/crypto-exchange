/// 应用常量定义

/// 最小用户ID - 小于此值的用户不能被迁移
pub const MIN_USER_ID: i64 = 10000;

/// SA Token related constants
pub const SA_TOKEN_KEY_PREFIX: &str = "sa-token:";
pub const SA_TOKEN_AUTH_HEADER_NAME: &str = "Authorization";

pub const ADMIN_INFO_KEY_AGENT_TYPE: &str = "agent_type";
/// Redis Keys
pub mod redis_keys {
    /// 代理迁移进行中锁
    pub const AGENT_MOVE_ING: &str = "agentMoveIng";
    
    pub const DEFAULT_AVATAR: &str = "/default_avatar.jpg";

    /// 顶级代理映射缓存
    pub const TOP_AGENT_MAP_CACHE_KEY: &str = "top_agent_maps";
}
