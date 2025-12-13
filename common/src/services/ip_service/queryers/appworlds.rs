use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct AppworldsQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct AppworldsResponse {
    code: i32,
    data: Option<AppworldsData>,
}

#[derive(Deserialize)]
struct AppworldsData {
    #[serde(rename = "fullAddress")]
    full_address: String,
}

impl AppworldsQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for AppworldsQueryer {
    fn support_ipv6(&self) -> bool {
        false
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://ip.appworlds.cn/?ip={}", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Appworlds request failed: {}", e)})))?;
        
        let data: AppworldsResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Appworlds response: {}", e)})))?;

        if data.code != 200 {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Appworlds query failed: code={}", data.code)})));
        }

        if let Some(d) = data.data {
            Ok(d.full_address)
        } else {
            Err(AppError::business("error.external_api"))
        }
    }
}
