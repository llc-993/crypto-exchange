use serde::{Deserialize, Serialize};
use config_derive::ConfigMapping;

/// EMQX配置
#[derive(Debug, Clone, Serialize, Deserialize, ConfigMapping)]
#[serde(rename_all = "camelCase")]
pub struct EmqxConfig {
    /// 是否启用emqx
    #[config(code = "emqxEnable", default = "false")]
    pub emqx_enable: Option<String>,
    /// EMQX API域名
    #[config(code = "emqxApiHost")]
    pub emqx_api_host: Option<String>,
    /// EMQX API端口
    #[config(code = "emqxApiPort")]
    pub emqx_api_port: Option<String>,
    /// EMQX API Key
    #[config(code = "emqxApiKey")]
    pub emqx_api_key: Option<String>,
    /// EMQX API Secret
    #[config(code = "emqxApiSecret")]
    pub emqx_api_secret: Option<String>,
}
