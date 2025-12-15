use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct IpCnQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct IpCnResponse {
    code: i32,
    address: Option<String>,
}

impl IpCnQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for IpCnQueryer {
    fn support_ipv6(&self) -> bool {
        false
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://www.ip.cn/api/index?ip={}&type=1", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpCn request failed: {}", e)})))?;
        
        let data: IpCnResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse IpCn response: {}", e)})))?;

        if data.code != 0 {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpCn query failed: code={}", data.code)})));
        }

        data.address.ok_or_else(|| AppError::business("error.external_api"))
    }
}
