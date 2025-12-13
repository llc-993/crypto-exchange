use serde::{Deserialize, Serialize};

/// Field mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub field_name: String,
    pub config_code: String,
    pub default_value: Option<String>,
}

/// Trait for configuration mapping
pub trait ConfigMapping: Default + serde::de::DeserializeOwned + serde::Serialize {
    fn field_mappings() -> Vec<FieldMapping>;
    fn from_config_map(map: std::collections::HashMap<String, String>) -> Self;
    fn cache_key() -> String;
}
