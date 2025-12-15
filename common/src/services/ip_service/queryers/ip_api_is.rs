use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct IpApiIsQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct IpApiIsResponse {
    location: Option<IpApiIsLocation>,
}

#[derive(Deserialize)]
struct IpApiIsLocation {
    country: Option<String>,
    city: Option<String>,
}

impl IpApiIsQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for IpApiIsQueryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://api.ipapi.is/?ip={}", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpApiIs request failed: {}", e)})))?;
        
        let data: IpApiIsResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse IpApiIs response: {}", e)})))?;

        if let Some(loc) = data.location {
            let country = loc.country.unwrap_or_default();
            let city = loc.city.unwrap_or_default();
            Ok(format!("{} {}", country, city).trim().to_string())
        } else {
            Err(AppError::business("error.external_api"))
        }
    }
}
