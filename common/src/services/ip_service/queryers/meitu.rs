use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

pub struct MeituQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct MeituResponse {
    code: i32,
    data: Option<HashMap<String, MeituData>>,
}

#[derive(Deserialize)]
struct MeituData {
    nation: Option<String>,
    city: Option<String>,
}

impl MeituQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for MeituQueryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://webapi-pc.meitu.com/common/ip_location?ip={}", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Meitu request failed: {}", e)})))?;
        
        let data: MeituResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Meitu response: {}", e)})))?;

        if data.code != 0 {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Meitu query failed: code={}", data.code)})));
        }

        if let Some(map) = data.data {
            if let Some(info) = map.get(ip) {
                let nation = info.nation.clone().unwrap_or_default();
                let city = info.city.clone().unwrap_or_default();
                return Ok(format!("{} {}", nation, city).trim().to_string());
            }
        }
        
        Err(AppError::business("error.external_api"))
    }
}
