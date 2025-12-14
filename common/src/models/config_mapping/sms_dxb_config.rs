use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;

/// 短信宝配置
#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct SmsDXBConfig {
    /// 短信宝账号
    #[config(code = "duanxinbaoUsername")]
    pub duanxinbao_username: Option<String>,
    /// 短信宝API Key
    #[config(code = "duanxinbaoApikey")]
    pub duanxinbao_apikey: Option<String>,
}
