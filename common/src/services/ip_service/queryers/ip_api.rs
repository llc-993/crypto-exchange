use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// IpApi.com 查询器
pub struct IpApiQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct IpApiResponse {
    status: String,
    country: Option<String>,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
}

impl IpApiQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for IpApiQueryer {
    fn support_ipv6(&self) -> bool {
        true
    }
    
    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("http://ip-api.com/json/{}?lang=zh-CN", ip);
        
        log::debug!("Querying IP geolocation: {}", url);
        
        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpApi request failed: {}", e)})))?;
        
        let data: IpApiResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse IpApi response: {}", e)})))?;
        
        if data.status != "success" {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("IpApi query failed: status={}", data.status)})));
        }
        
        let country = data.country.unwrap_or_default();
        let city = data.city.unwrap_or_default();
        let region = data.region_name.unwrap_or_default();
        
        let address = format!("{} {} {}", country, city, region).trim().to_string();
        
        log::info!("IP {} located at: {}", ip, address);
        
        Ok(address)
    }
}
