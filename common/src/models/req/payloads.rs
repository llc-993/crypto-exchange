use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredPayload {
    pub user_id: u64,
    pub username: String,
    pub ip: Option<String>,
}
