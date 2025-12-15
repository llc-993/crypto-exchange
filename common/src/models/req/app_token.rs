use serde::Deserialize;

// DTO for login
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub user_account: String,
    pub password: String,
    #[serde(default = "default_remember_me")]
    pub remember_me: bool,
}

fn default_remember_me() -> bool { true }

