use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use sa_token_core::{SaTokenContext, token::TokenValue};
use sa_token_plugin_actix_web::SaTokenState;
use crate::error::AppError;
use super::auth_checker::AuthChecker;

/// 自定义 Sa-Token 中间件 - 自己实现 call 方法的拦截逻辑
#[derive(Clone)]
pub struct SaTokenMiddleware {
    pub state: SaTokenState,
    pub auth_checker: Arc<dyn AuthChecker>,
}


impl SaTokenMiddleware {
    pub fn new(state: SaTokenState, auth_checker: Arc<dyn AuthChecker>) -> Self {
        Self {
            state,
            auth_checker,
        }
    }

    /// 创建一个构建器
    pub fn builder() -> SaTokenMiddlewareBuilder {
        SaTokenMiddlewareBuilder::new()
    }
}

/// SaTokenMiddleware 构建器
pub struct SaTokenMiddlewareBuilder {
    state: Option<SaTokenState>,
    auth_checker: Option<Arc<dyn AuthChecker>>,
}

impl SaTokenMiddlewareBuilder {
    pub fn new() -> Self {
        Self {
            state: None,
            auth_checker: None,
        }
    }

    /// 设置 SaTokenState (必须)
    pub fn state(mut self, state: SaTokenState) -> Self {
        self.state = Some(state);
        self
    }

    /// 设置 AuthChecker (必须)
    pub fn auth_checker(mut self, auth_checker: Arc<dyn AuthChecker>) -> Self {
        self.auth_checker = Some(auth_checker);
        self
    }

    /// 构建 SaTokenMiddleware
    /// 
    /// # Panics
    /// 如果 `state` 或 `auth_checker` 未设置，则 panic。
    pub fn build(self) -> SaTokenMiddleware {
        SaTokenMiddleware {
            state: self.state.expect("SaTokenMiddlewareBuilder: state is required"),
            auth_checker: self.auth_checker.expect("SaTokenMiddlewareBuilder: auth_checker is required"),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for SaTokenMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SaTokenMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SaTokenMiddlewareService {
            service: Rc::new(service),
            state: self.state.clone(),
            auth_checker: self.auth_checker.clone(),
        }))
    }
}

pub struct SaTokenMiddlewareService<S> {
    service: Rc<S>,
    state: SaTokenState,
    auth_checker: Arc<dyn AuthChecker>,
}

impl<S, B> Service<ServiceRequest> for SaTokenMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {

        let service = Rc::clone(&self.service);
        let state = self.state.clone();
        let auth_checker = self.auth_checker.clone();

        Box::pin(async move {
            let mut ctx = SaTokenContext::new();

            // 1. 判断是否需要鉴权
            let need_auth = auth_checker.check_auth_required(&req);
            
            // 2. 尝试提取 Token
            let token_str_opt = extract_token_from_request(&req, &state);
            
            let is_token_none = token_str_opt.is_none();

            // 如果未提供token,但是需要鉴权，直接阻止
            if is_token_none && need_auth {
                log::warn!("⚠️  [Auth] 未提供 Token，且接口需要鉴权");
                return Err(AppError::auth("error.token_missing").into());
            }

            // 3. 如果未提供 Token
            if is_token_none {
                // 未提供token,也不需要鉴权，放行。（不清楚是否需要设置上下文，保险起见还是保留）
                // 4. 设置上下文并继续处理
                SaTokenContext::set_current(ctx);
                let result = service.call(req).await;
                SaTokenContext::clear();
                return result
            }
            // 代码执行到这里，这里一定存在token_str
            // 4. 如果提供了 Token，进行验证，
            let token_str = token_str_opt.unwrap();

            let token = TokenValue::new(token_str);

            // 验证 Token
            let token_valid = state.manager.is_valid(&token).await;
            if !token_valid && need_auth {
                // 需要鉴权，但是token无效。直接阻止
                log::warn!("⚠️  [Auth] Token 无效或已过期，且接口需要鉴权");
                return Err(AppError::auth("error.token_invalid").into());
            }

            if token_valid {
                // Token 有效
                log::debug!("✅ [Auth] Token 验证通过");

                // 存储 Token 和 LoginId 到请求扩展
                req.extensions_mut().insert(token.clone());

                if let Ok(token_info) = state.manager.get_token_info(&token).await {
                    let login_id = token_info.login_id.clone();
                    
                    if !auth_checker.valid_login_id(login_id.as_str()) {
                        // 需要鉴权，但是login_id无效。直接阻止
                        log::warn!("⚠️  [Auth] Login_id 无效: {}", &login_id);
                        return Err(AppError::auth("error.token_invalid").into());
                    }
                    
                    req.extensions_mut().insert(login_id.clone());

                    // 设置上下文
                    ctx.token = Some(token.clone());
                    ctx.token_info = Some(Arc::new(token_info));
                    ctx.login_id = Some(login_id);
                }
            }

            // 4. 设置上下文并继续处理
            SaTokenContext::set_current(ctx);
            let result = service.call(req).await;
            SaTokenContext::clear();
            
            result
        })
    }
}

/// 从请求中提取 token (自定义实现)
fn extract_token_from_request(req: &ServiceRequest, state: &SaTokenState) -> Option<String> {
    let token_name = &state.manager.config.token_name;

    log::debug!("🔍 [Auth] 尝试提取 token，token_name: {}", token_name);

    // 1. 优先从 Header 中获取
    if let Some(auth_header) = req.headers().get(token_name) {
        if let Ok(auth_str) = auth_header.to_str() {
            log::debug!("📋 [Auth] 从 Header[{}] 获取到 token", token_name);
            return Some(extract_bearer_token(auth_str));
        }
    }

    // 2. 如果 token_name 不是 "Authorization"，也尝试从 "Authorization" 头获取
    if token_name != "Authorization" {
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                log::debug!("📋 [Auth] 从 Header[Authorization] 获取到 token");
                return Some(extract_bearer_token(auth_str));
            }
        }
    }

    // 3. 从 Cookie 中获取
    if let Some(cookie) = req.cookie(token_name) {
        log::debug!("🍪 [Auth] 从 Cookie[{}] 获取到 token", token_name);
        return Some(cookie.value().to_string());
    }

    // 4. 从 Query 参数中获取
    if let Some(query) = req.query_string().split('&').find_map(|pair| {
        let mut parts = pair.split('=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            if key == token_name {
                // 简单的 URL 解码
                return Some(value.replace("%20", " ").to_string());
            }
        }
        None
    }) {
        log::debug!("🔗 [Auth] 从 Query[{}] 获取到 token", token_name);
        return Some(query);
    }

    log::debug!("❌ [Auth] 所有位置都未找到 token");
    None
}

/// 提取 Bearer token
fn extract_bearer_token(token: &str) -> String {
    if token.starts_with("Bearer ") {
        token[7..].to_string()
    } else {
        token.to_string()
    }
}
