use serde::Serialize;

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}