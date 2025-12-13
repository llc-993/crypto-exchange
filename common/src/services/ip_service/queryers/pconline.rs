use crate::error::AppError;
use crate::services::ip_service::queryer::IpQueryer;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct PconlineQueryer {
    client: Client,
}

#[derive(Deserialize)]
struct PconlineResponse {
    addr: Option<String>,
}

impl PconlineQueryer {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

#[async_trait]
impl IpQueryer for PconlineQueryer {
    fn support_ipv6(&self) -> bool {
        false
    }

    async fn get_real_address(&self, ip: &str) -> Result<String, AppError> {
        let url = format!("http://whois.pconline.com.cn/ipJson.jsp?ip={}&json=true", ip);
        
        let resp = self.client.get(&url).send().await
            .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Pconline request failed: {}", e)})))?;
        
        let text = resp.text().await
             .map_err(|e| AppError::business_with_params("error.external_api", serde_json::json!({"msg": format!("Failed to get text from Pconline: {}", e)})))?;
             
        // Attempt to clean up if it's not pure JSON (sometimes they wrap in callbacks or have whitespace)
        let text = text.trim();
        
        let data: PconlineResponse = serde_json::from_str(text)
            .map_err(|e| AppError::unknown_with_params("error.parse_error", serde_json::json!({"msg": format!("Failed to parse Pconline response: {}", e)})))?;

        data.addr.ok_or_else(|| AppError::business("error.external_api"))
    }
}
