/// Serde 序列化辅助函数
/// 
/// 提供常用的自定义序列化/反序列化功能

/// Rbatis DateTime 格式化模块
pub mod rbatis_datetime {
    use rbatis::rbdc::DateTime;
    use serde::Serializer;
    
    /// 将 Rbatis DateTime 序列化为字符串格式 "YYYY-MM-DD HH:MM:SS"
    /// 
    /// # Example (在 common crate 内部)
    /// ```
    /// use serde::Serialize;
    /// use rbatis::rbdc::DateTime;
    /// 
    /// #[derive(Serialize)]
    /// struct MyStruct {
    ///     #[serde(serialize_with = "crate::utils::serde_helpers::rbatis_datetime::serialize")]
    ///     pub create_time: Option<DateTime>,
    /// }
    /// ```
    pub fn serialize<S>(date: &Option<DateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => {
                // Rbatis DateTime 方法：year(), mon(), day(), hour(), minute(), sec()
                // 格式化为 YYYY-MM-DD HH:MM:SS
                let formatted = format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    dt.year(), dt.mon(), dt.day(), dt.hour(), dt.minute(), dt.sec()
                );
                serializer.serialize_str(&formatted)
            }
            None => serializer.serialize_none(),
        }
    }
}

/// 可选的时间戳格式化（Unix 时间戳）
pub mod unix_timestamp {
    use rbatis::rbdc::DateTime;
    use serde::Serializer;
    
    /// 将 DateTime 序列化为 Unix 时间戳（秒）
    pub fn serialize<S>(date: &Option<DateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(_dt) => {
                // 如果需要转换为时间戳，可以在这里实现
                // 目前 Rbatis DateTime 可能没有直接的 timestamp 方法
                // 可以根据需要扩展
                serializer.serialize_none()
            }
            None => serializer.serialize_none(),
        }
    }
}
