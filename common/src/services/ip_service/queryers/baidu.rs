use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct BaiduQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct BaiduResponse {
    status: String,
    data: Option<Vec<BaiduData>>,
}

#[derive(Deserialize)]
struct BaiduData {
    location: String,
}

impl BaiduQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for BaiduQueryer {
    fn support_ipv6(&self) -> bool {
        false
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://opendata.baidu.com/api.php?query={}&co=&resource_id=6006&oe=utf8", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Baidu request failed: {}", e)})))?;
        
        // Baidu API might return non-standard JSON or encoding issues, but reqwest handles utf-8
        let data: BaiduResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Baidu response: {}", e)})))?;

        if data.status != "0" {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Baidu query failed: status={}", data.status)})));
        }

        if let Some(items) = data.data {
            if let Some(item) = items.first() {
                return Ok(item.location.clone());
            }
        }
        
        Err(AppError::business("error.external_api"))
    }
}
