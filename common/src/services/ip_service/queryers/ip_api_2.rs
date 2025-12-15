use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct IpApi2Queryer {
    client: Client,
}

#[derive(Deserialize)]
struct IpApi2Response {
    status: String,
    country: Option<String>,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
}

impl IpApi2Queryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for IpApi2Queryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("http://demo.ip-api.com/json/{}?lang=zh-CN", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpApi2 request failed: {}", e)})))?;
        
        let data: IpApi2Response = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse IpApi2 response: {}", e)})))?;

        if data.status != "success" {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpApi2 query failed: status={}", data.status)})));
        }

        let country = data.country.unwrap_or_default();
        let city = data.city.unwrap_or_default();
        let region = data.region_name.unwrap_or_default();
        
        Ok(format!("{} {} {}", country, city, region).trim().to_string())
    }
}
