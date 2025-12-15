use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct VoreQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct VoreResponse {
    code: i32,
    adcode: Option<VoreAdCode>,
}

#[derive(Deserialize)]
struct VoreAdCode {
    o: Option<String>,
}

impl VoreQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for VoreQueryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://api.vore.top/api/IPdata?ip={}", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Vore request failed: {}", e)})))?;
        
        let data: VoreResponse = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Vore response: {}", e)})))?;

        if data.code != 200 {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Vore query failed: code={}", data.code)})));
        }

        if let Some(adcode) = data.adcode {
            adcode.o.ok_or_else(|| AppError::business("error.external_api"))
        } else {
            Err(AppError::business("error.external_api"))
        }
    }
}
