use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct QjqqQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct QjqqResponse {
    code: i32,
    data: Option<QjqqData>,
}

#[derive(Deserialize)]
struct QjqqData {
    country: Option<String>,
    city: Option<String>,
}

impl QjqqQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for QjqqQueryer {
    fn support_ipv6(&self) -> bool {
        false
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://api.qjqq.cn/api/district?ip={}", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Qjqq request failed: {}", e)})))?;
        
        let data: QjqqResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Qjqq response: {}", e)})))?;

        if data.code != 200 {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Qjqq query failed: code={}", data.code)})));
        }

        if let Some(d) = data.data {
            let country = d.country.unwrap_or_default();
            let city = d.city.unwrap_or_default();
            Ok(format!("{} {}", country, city).trim().to_string())
        } else {
            Err(AppError::business("error.external_api"))
        }
    }
}
