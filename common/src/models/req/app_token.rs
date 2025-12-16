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


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {

    pub user_account: String,

    pub password: String,

    pub money_password: String,

    pub invite_code: Option<String>,

}