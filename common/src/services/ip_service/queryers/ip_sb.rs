use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct IpSbQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct IpSbResponse {
    country: Option<String>,
    city: Option<String>,
}

impl IpSbQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for IpSbQueryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://api.ip.sb/geoip/{}", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpSb request failed: {}", e)})))?;
        
        let data: IpSbResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse IpSb response: {}", e)})))?;

        let country = data.country.unwrap_or_default();
        let city = data.city.unwrap_or_default();
        
        Ok(format!("{} {}", country, city).trim().to_string())
    }
}
