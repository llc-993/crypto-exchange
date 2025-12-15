use actix_web::dev::ServiceRequest;
use crate::error::AppError;

/// 路由匹配工具类 (参考 Sa-Token Java 版本)
/// 
/// 提供流式 API 进行路由匹配和拦截鉴权。
/// 
/// # 工作原理
/// 
/// `SaRouter` 维护一个 `is_hit` 状态，初始为 `true`。
/// - `match(pattern)`: 如果当前路径匹配 `pattern`，`is_hit` 保持不变；否则 `is_hit` 变为 `false`。
///   (注意：多个 `match` 串联时，相当于 AND 逻辑，即必须同时满足所有 match 条件)
///   **更正**: Java 版 `match` 是 "命中列表"，只要命中其中一个即可。但链式调用 `match(A).match(B)` 是什么语义？
///   Java 版 `SaRouter.match(patterns)` 是 `new SaRouterStaff().match(patterns)`.
///   `SaRouterStaff.match(patterns)` 逻辑是：如果当前 path 匹配 patterns 中的**任意一个**，则 `is_hit` 为 true (前提是之前也是 true?)。
///   如果不匹配，则 `is_hit` 为 false。
///   
///   通常用法是：`SaRouter.match("/**").notMatch("/public/**").check(...)`
///   1. `match("/**")`: 匹配，`is_hit` = true.
///   2. `notMatch("/public/**")`: 如果匹配排除列表，`is_hit` = false.
///   3. `check(...)`: 如果 `is_hit` 为 true，执行检查.
/// 
/// # 示例
/// 
/// ```rust
/// SaRouter::new(&req)
///     .match_path("/**")
///     .not_match_path("/api/public/**")
///     .check(|_| {
///         // 鉴权逻辑
///         Ok(())
///     })?;
/// ```
pub struct SaRouter {
    // 当前请求路径
    path: String,
    // 当前请求方法
    method: String,
    // 是否命中 (是否需要执行 check)
    is_hit: bool,
}

impl SaRouter {
    /// 创建新的路由匹配器
    pub fn new(req: &ServiceRequest) -> Self {
        SaRouter {
            path: req.path().to_string(),
            method: req.method().as_str().to_string(),
            is_hit: true, // 默认为 true，如果没有 match 调用，直接 check 会执行
        }
    }

    /// 仅用于测试或非 Request 场景
    pub fn new_unchecked(path: &str, method: &str) -> Self {
        SaRouter {
            path: path.to_string(),
            method: method.to_string(),
            is_hit: true,
        }
    }

    /// 获取当前命中状态
    pub fn is_hit(&self) -> bool {
        self.is_hit
    }

    // ------------------- 匹配方法 -------------------

    /// 匹配路径 (白名单)
    /// 
    /// 如果当前路径匹配传入的模式(或模式列表中的任意一个)，则保持命中状态；
    /// 否则，标记为未命中。
    /// 
    /// # 参数
    /// `patterns`: 可以是单个 &str，也可以是 Vec<&str> 或 &[&str]
    pub fn match_path<P>(mut self, patterns: P) -> Self 
    where P: IntoPatterns 
    {
        if !self.is_hit {
            return self;
        }

        let patterns = patterns.into_patterns();
        let mut matched = false;
        for pattern in patterns {
            if self.is_match(&pattern) {
                matched = true;
                break;
            }
        }

        if !matched {
            self.is_hit = false;
        }
        self
    }

    /// 排除路径 (黑名单)
    /// 
    /// 如果当前路径匹配传入的模式(或模式列表中的任意一个)，则标记为未命中。
    pub fn not_match_path<P>(mut self, patterns: P) -> Self 
    where P: IntoPatterns 
    {
        if !self.is_hit {
            return self;
        }

        let patterns = patterns.into_patterns();
        for pattern in patterns {
            if self.is_match(&pattern) {
                self.is_hit = false;
                break;
            }
        }
        self
    }

    /// 匹配 HTTP 方法
    pub fn match_method<P>(mut self, methods: P) -> Self
    where P: IntoPatterns
    {
        if !self.is_hit {
            return self;
        }
        
        let methods = methods.into_patterns();
        let mut matched = false;
        for method in methods {
            if self.method.eq_ignore_ascii_case(&method) {
                matched = true;
                break;
            }
        }

        if !matched {
            self.is_hit = false;
        }
        self
    }

    /// 排除 HTTP 方法
    pub fn not_match_method<P>(mut self, methods: P) -> Self
    where P: IntoPatterns
    {
        if !self.is_hit {
            return self;
        }

        let methods = methods.into_patterns();
        for method in methods {
            if self.method.eq_ignore_ascii_case(&method) {
                self.is_hit = false;
                break;
            }
        }
        self
    }

    // ------------------- 执行方法 -------------------

    /// 执行校验函数
    /// 
    /// 如果当前状态为命中 (`is_hit == true`)，则执行传入的闭包。
    pub fn check<F>(self, f: F) -> Result<Self, AppError>
    where
        F: FnOnce(&Self) -> Result<(), AppError>,
    {
        if self.is_hit {
            f(&self)?;
        }
        Ok(self)
    }

    /// 停止匹配 (模拟 Java 的 stop)
    /// 
    /// 直接将 is_hit 置为 false，后续的 check 将不会执行。
    pub fn stop(mut self) -> Self {
        self.is_hit = false;
        self
    }

    // ------------------- 内部工具 -------------------

    /// 判断路径是否匹配 (Ant 风格)
    fn is_match(&self, pattern: &str) -> bool {
        if pattern == "/**" {
            return true;
        }
        
        // 简单通配符支持
        // TODO: 引入 wildmatch crate 或增强实现以支持更复杂的 Ant 风格
        // 目前支持:
        // - /** : 匹配所有
        // - /xxx/** : 前缀匹配
        // - *.html : 后缀匹配
        // - /xxx/* : 单级目录匹配 (简单前缀)

        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return self.path.starts_with(prefix);
        }

        if pattern.starts_with("*") {
             let suffix = &pattern[1..];
             return self.path.ends_with(suffix);
        }

        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            if self.path.starts_with(prefix) {
                let suffix = &self.path[prefix.len()..];
                // 确保不包含多级目录
                return !suffix.contains('/') || suffix == "/";
            }
            return false;
        }

        self.path == pattern
    }
}

// ------------------- 辅助 Trait -------------------

/// 辅助 trait，用于支持传入单个 &str 或 Vec<&str>
pub trait IntoPatterns {
    fn into_patterns(self) -> Vec<String>;
}

impl IntoPatterns for &str {
    fn into_patterns(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IntoPatterns for String {
    fn into_patterns(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoPatterns for Vec<&str> {
    fn into_patterns(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl IntoPatterns for Vec<String> {
    fn into_patterns(self) -> Vec<String> {
        self
    }
}

impl<const N: usize> IntoPatterns for [&str; N] {
    fn into_patterns(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}
