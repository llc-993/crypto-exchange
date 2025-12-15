use std::sync::Arc;
use reqwest::Client;
use log::{info, warn, error};
use crate::error::AppError;
use crate::models::req::sms_req::SendSmsReq;
use crate::models::config_mapping::sms_dxb_config::SmsDXBConfig;
use crate::services::config_service::ConfigService;
use super::SmsService;

/// 短信宝SMS服务实现
pub struct DXBSmsService {
    config_service: Arc<ConfigService>,
    client: Client,
}

impl DXBSmsService {
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        let client = Client::new();
        Self { config_service, client }
    }
}

#[async_trait::async_trait]
impl SmsService for DXBSmsService {
    fn name(&self) -> &str {
        "dxb"
    }

    async fn send(&self, req: SendSmsReq) -> Result<bool, AppError> {
        // 加载短信宝配置
        let config = self.config_service.load_config::<SmsDXBConfig>().await?;
        
        let username = config.duanxinbao_username
            .ok_or_else(|| AppError::unknown("error.sms_username_not_configured"))?;
        let apikey = config.duanxinbao_apikey
            .ok_or_else(|| AppError::unknown("error.sms_apikey_not_configured"))?;
        
        // 处理手机号：添加+前缀，然后替换为%2b
        let mut mobile = if !req.mobile.starts_with('+') {
            format!("+{}", req.mobile)
        } else {
            req.mobile.clone()
        };
        mobile = mobile.replace('+', "%2b");
        
        // URL编码内容
        let content = urlencoding::encode(&req.content);
        
        // 构建请求URL
        let url = format!(
            "https://api.smsbao.com/wsms?u={}&p={}&m={}&c={}",
            username, apikey, mobile, content
        );
        
        info!("发送短信: 手机号={}, 内容={}", req.mobile, req.content);
        
        // 发送HTTP GET请求
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("短信发送请求失败: {:?}", e);
                AppError::unknown_with_params(
                    "error.sms_request_failed",
                    serde_json::json!({"msg": e.to_string()})
                )
            })?
            .text()
            .await
            .map_err(|e| {
                error!("读取短信响应失败: {:?}", e);
                AppError::unknown_with_params(
                    "error.sms_response_failed",
                    serde_json::json!({"msg": e.to_string()})
                )
            })?;
        
        info!("短信发送响应: 手机号={}, 内容={}, 响应码={}", req.mobile, req.content, resp);
        
        // 处理响应码
        match resp.trim() {
            "0" => {
                info!("短信发送成功");
                Ok(true)
            }
            "30" => {
                error!("短信发送失败: 错误密码");
                Err(AppError::business("error.sms_wrong_password"))
            }
            "41" => {
                error!("短信发送失败: 余额不足");
                Err(AppError::business("error.sms_insufficient_balance"))
            }
            "43" => {
                error!("短信发送失败: IP地址限制");
                Err(AppError::business("error.sms_ip_restricted"))
            }
            "50" => {
                error!("短信发送失败: 内容含有敏感词");
                Err(AppError::business("error.sms_sensitive_content"))
            }
            "51" => {
                error!("短信发送失败: 手机号码不正确");
                Err(AppError::validation("error.sms_invalid_phone"))
            }
            code => {
                warn!("短信发送返回未知状态码: {}", code);
                Ok(false)
            }
        }
    }
}
