use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use std::future::{ready, Ready};
use futures::future::LocalBoxFuture;

pub struct I18n;

impl<S, B> Transform<S, ServiceRequest> for I18n
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = I18nMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(I18nMiddleware { service }))
    }
}

pub struct I18nMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for I18nMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 按优先级获取语言设置
        let locale = get_locale_from_request(&req);
        
        // 设置当前请求的语言
        rust_i18n::set_locale(&locale);
        
        // 将语言信息保存到 extensions 中，供后续使用
        req.extensions_mut().insert(locale.clone());
        
        let fut = self.service.call(req);
        
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

/// 按优先级从请求中获取语言设置
/// 1. 查询参数 ?lang=zh-CN
/// 2. HTTP Header Accept-Language
/// 3. Cookie lang
/// 4. 默认语言 en
fn get_locale_from_request(req: &ServiceRequest) -> String {
    // 1. 从查询参数获取
    if let Some(lang) = extract_lang_from_query(req.query_string()) {
        return normalize_locale(&lang);
    }
    
    // 2. 从 Accept-Language header 获取
    if let Some(lang) =  req.headers().get("Lang") {
        if let Ok(lang_str) = lang.to_str() {
            return normalize_locale(lang_str);
        }
    }
    if let Some(lang) = req.headers().get("Accept-Language") {
        if let Ok(lang_str) = lang.to_str() {
            // 解析 Accept-Language 格式: zh-CN,zh;q=0.9,en;q=0.8
            if let Some(first_lang) = lang_str.split(',').next() {
                let locale = first_lang.split(';').next().unwrap_or("en").trim();
                if !locale.is_empty() {
                    return normalize_locale(locale);
                }
            }
        }
    }
    
    // 3. 从 Cookie 获取（如果需要）
    // if let Some(cookie) = req.cookie("lang") {
    //     return normalize_locale(cookie.value());
    // }
    
    // 4. 返回默认语言
    "en".to_string()
}

/// 规范化 locale 字符串，将 '-' 替换为 '_'
/// 例如: zh-CN -> zh_CN, en-US -> en_US
fn normalize_locale(locale: &str) -> String {
    locale.replace('-', "_")
}

/// 从查询字符串中提取 lang 参数
fn extract_lang_from_query(query: &str) -> Option<String> {
    query.split('&').find_map(|param| {
        let mut parts = param.split('=');
        if parts.next()? == "lang" {
            Some(parts.next()?.to_string())
        } else {
            None
        }
    })
}
