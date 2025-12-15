use std::sync::Arc;
use reqwest::Client;
use base64::{Engine as _, engine::general_purpose};
use log::{info, warn, error};
use crate::error::AppError;
use crate::models::req::mqtt_msg::MqttMsg;
use crate::models::config_mapping::emqx_config::EmqxConfig;
use crate::services::config_service::ConfigService;

/// EMQX服务
/// 
/// 提供MQTT消息发布功能
pub struct EmqxService {
    config_service: Arc<ConfigService>,
    client: Client,
}

impl EmqxService {
    const API_PUBLISH_PATH: &'static str = "/api/v5/publish";

    /// 创建新的EMQX服务实例
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        let client = Client::new();
        Self {
            config_service,
            client,
        }
    }

    /// 发布MQTT消息（异步，非阻塞）
    pub fn publish(&self, msg: MqttMsg) {
        let config_service = self.config_service.clone();
        let client = self.client.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::do_publish(config_service, client, msg).await {
                error!("EMQX发布消息失败: {:?}", e);
            }
        });
    }

    /// 执行消息发布
    async fn do_publish(
        config_service: Arc<ConfigService>,
        client: Client,
        msg: MqttMsg,
    ) -> Result<(), AppError> {
        // 检查是否启用EMQX
        let enable_value = config_service
            .get_value_by_code("emqxEnable", Some("false"))
            .await?
            .unwrap_or_else(|| "false".to_string());

        if enable_value != "true" {
            warn!("未开启 emqxEnable");
            return Ok(());
        }

        // 加载EMQX配置
        let config = config_service.load_config::<EmqxConfig>().await?;

        let api_host = config.emqx_api_host
            .ok_or_else(|| AppError::unknown("error.emqx_api_host_not_configured"))?;
        let api_port = config.emqx_api_port
            .ok_or_else(|| AppError::unknown("error.emqx_api_port_not_configured"))?;
        let api_key = config.emqx_api_key
            .ok_or_else(|| AppError::unknown("error.emqx_api_key_not_configured"))?;
        let api_secret = config.emqx_api_secret
            .ok_or_else(|| AppError::unknown("error.emqx_api_secret_not_configured"))?;

        // 构建API URL
        let api_url = format!("{}:{}{}", api_host, api_port, Self::API_PUBLISH_PATH);

        // 构建Basic Auth
        let auth_string = format!("{}:{}", api_key, api_secret);
        let basic_auth = format!("Basic {}", general_purpose::STANDARD.encode(auth_string.as_bytes()));

        // 发送HTTP POST请求
        let resp = client
            .post(&api_url)
            .header("Authorization", basic_auth)
            .header("Content-Type", "application/json")
            .json(&msg)
            .send()
            .await
            .map_err(|e| AppError::unknown_with_params(
                "error.emqx_request_failed",
                serde_json::json!({"msg": e.to_string()})
            ))?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| String::from(""));

        // HTTP 200 和 202 都视为成功
        if status.is_success() || status.as_u16() == 202 {
            info!("EMQX消息发布成功: topic={}, status={}, resp={}", msg.topic, status, body);
            Ok(())
        } else {
            error!("EMQX消息发布失败: topic={}, status={}, resp={}", msg.topic, status, body);
            Err(AppError::unknown_with_params(
                "error.emqx_publish_failed",
                serde_json::json!({"status": status.as_u16(), "body": body})
            ))
        }
    }
}
