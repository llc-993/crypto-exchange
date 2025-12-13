use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct IpApiCoQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct IpApiCoResponse {
    error: Option<bool>,
    country_name: Option<String>,
    city: Option<String>,
}

impl IpApiCoQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for IpApiCoQueryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://ipapi.co/{}/json/", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpApiCo request failed: {}", e)})))?;
        
        let data: IpApiCoResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse IpApiCo response: {}", e)})))?;

        if let Some(true) = data.error {
            return Err(AppError::business("error.external_api"));
        }

        let country = data.country_name.unwrap_or_default();
        let city = data.city.unwrap_or_default();
        
        Ok(format!("{} {}", country, city).trim().to_string())
    }
}
