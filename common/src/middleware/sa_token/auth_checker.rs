use actix_web::dev::ServiceRequest;
use super::sa_router::SaRouter;

/// 认证检查器 trait
/// 
/// 用于判断请求是否需要进行身份认证
pub trait AuthChecker: Send + Sync {
    /// 检查请求是否需要鉴权
    /// 
    /// # 参数
    /// - `vo`: 当前请求
    /// 
    /// # 返回
    /// - `true`: 需要鉴权
    /// - `false`: 不需要鉴权
    fn check_auth_required(&self, req: &ServiceRequest) -> bool;

    /// 验证 LoginId
    ///
    /// # 参数
    /// - `vo`: 当前请求
    /// - `_login_id`: 登录ID
    ///
    /// # 返回
    /// - `true`: 通过
    /// - `false`: 不通过
    fn valid_login_id(&self, _login_id: &str) -> bool {
        true
    }
}

use std::sync::Arc;

/// 基于 SaRouter 的默认认证检查器
pub struct DefaultAuthChecker {
    match_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    check_login_id_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl DefaultAuthChecker {
    pub fn new(match_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Self {
        Self {
            match_patterns,
            exclude_patterns,
            check_login_id_fn: None,
        }
    }

    /// 创建一个构建器
    pub fn builder() -> AuthCheckerBuilder {
        AuthCheckerBuilder::new()
    }
}

impl AuthChecker for DefaultAuthChecker {
    fn check_auth_required(&self, req: &ServiceRequest) -> bool {
        let router = SaRouter::new(req);
        let match_patterns: Vec<&str> = self.match_patterns.iter().map(|s| s.as_str()).collect();
        let exclude_patterns: Vec<&str> = self.exclude_patterns.iter().map(|s| s.as_str()).collect();
        
        router
            .match_path(match_patterns)
            .not_match_path(exclude_patterns)
            .is_hit()
    }

    fn valid_login_id(&self, login_id: &str) -> bool {
        if let Some(ref check_fn) = self.check_login_id_fn {
            (check_fn)(login_id)
        } else {
            true
        }
    }
}

/// AuthChecker 构建器
pub struct AuthCheckerBuilder {
    match_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    check_login_id_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl AuthCheckerBuilder {
    pub fn new() -> Self {
        Self {
            match_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            check_login_id_fn: None,
        }
    }

    /// 添加匹配路径 (单个)
    pub fn add_match(mut self, pattern: impl Into<String>) -> Self {
        self.match_patterns.push(pattern.into());
        self
    }

    /// 添加匹配路径 (多个)
    pub fn add_matches(mut self, patterns: Vec<String>) -> Self {
        self.match_patterns.extend(patterns);
        self
    }

    /// 添加排除路径 (单个)
    pub fn add_exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// 添加排除路径 (多个)
    pub fn add_excludes(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns.extend(patterns);
        self
    }
    
    /// 设置 LoginId 检查函数
    pub fn check_login_id<F>(mut self, check_fn: F) -> Self 
    where F: Fn(&str) -> bool + Send + Sync + 'static 
    {
        self.check_login_id_fn = Some(Arc::new(check_fn));
        self
    }

    /// 构建 DefaultAuthChecker
    pub fn build(self) -> DefaultAuthChecker {
        DefaultAuthChecker {
            match_patterns: self.match_patterns,
            exclude_patterns: self.exclude_patterns,
            check_login_id_fn: self.check_login_id_fn,
        }
    }
}
