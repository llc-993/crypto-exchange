use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct Mir6Queryer {
    client: Client,
}

#[derive(Deserialize)]
struct Mir6Response {
    code: i32,
    data: Option<Mir6Data>,
}

#[derive(Deserialize)]
struct Mir6Data {
    location: String,
}

impl Mir6Queryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for Mir6Queryer {
    fn support_ipv6(&self) -> bool {
        true
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("https://api.mir6.com/api/ip?ip={}&type=json", ip);
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Mir6 request failed: {}", e)})))?;
        
        let data: Mir6Response = resp.json().await
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Mir6 response: {}", e)})))?;

        if data.code != 200 {
            return Err(AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Mir6 query failed: code={}", data.code)})));
        }

        if let Some(d) = data.data {
            Ok(d.location)
        } else {
            Err(AppError::business("error.external_api"))
        }
    }
}
