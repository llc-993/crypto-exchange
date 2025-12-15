use crate::error::AppError;
use crate::utils::redis_util::RedisUtil;

use rbatis::{crud, RBatis};
use std::collections::HashMap;
use std::sync::Arc;
use rbdc::DateTime;
use serde::{Deserialize, Serialize};
use crate::services::config_mapping::ConfigMapping;

/// app配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub id: Option<i64>,
    pub code: Option<String>,
    pub value: Option<String>,
    pub remark: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

crud!(AppConfig {}, "app_config");

/// 配置服务
/// 
/// 提供配置的加载、设置和缓存管理功能
pub struct ConfigService {
    rb: Arc<RBatis>,
    redis: Arc<RedisUtil>,
}

impl ConfigService {
    /// 创建新的配置服务实例
    pub fn new(rb: Arc<RBatis>, redis: Arc<RedisUtil>) -> Self {
        Self { rb, redis }
    }

    /// Load configuration of type T from cache or database
    pub async fn load_config<T: ConfigMapping>(&self) -> Result<T, AppError> {
        let cache_key = T::cache_key();
        
        // 1. Try to get from Redis cache
        if let Ok(Some(cached_json)) = self.redis.get(&cache_key).await {
            if let Ok(loaded_config) = serde_json::from_str::<T>(&cached_json) {
                log::debug!("📦 Config loaded from cache: {}", cache_key);
                return Ok(loaded_config);
            }
        }
        
        log::debug!("🔍 Cache miss for {}, loading from database", cache_key);
        
        // 2. Load from database
        let configs = AppConfig::select_all(self.rb.as_ref())
            .await
            .map_err(|e| {
                AppError::unknown_with_params(
                    "error.db_query",
                    serde_json::json!({"msg": e.to_string()})
                )
            })?;
        
        // 3. Build config map
        let mut config_map: HashMap<String, String> = HashMap::new();
        for c in configs {
            if let (Some(code), Some(value)) = (c.code, c.value) {
                config_map.insert(code, value);
            }
        }
        
        // 4. Use ConfigMapping to build typed config
        let loaded_config = T::from_config_map(config_map);
        
        // 5. Cache for 1 hour
        let config_json = serde_json::to_string(&loaded_config)
            .map_err(|e| AppError::unknown_with_params(
                "error.serialization",
                serde_json::json!({"msg": e.to_string()})
            ))?;
        
        if let Err(e) = self.redis.set_ex(&cache_key, &config_json, 3600).await {
            log::warn!("Failed to cache config: {}", e);
        } else {
            log::debug!("💾 Config cached: {} (TTL: 3600s)", cache_key);
        }
        
        Ok(loaded_config)
    }

    /// Invalidate cached configuration
    pub async fn invalidate_config_cache(&self) -> Result<(), AppError> {
        // Delete all app_config:* keys using pattern matching
        let deleted = self.redis.del_pattern("app_config:*").await
            .map_err(|e| AppError::unknown_with_params(
                "error.redis",
                serde_json::json!({"msg": e.to_string()})
            ))?;
        
        log::info!("🗑️  Config cache invalidated ({} keys deleted)", deleted);
        Ok(())
    }

    /// Set configuration from ConfigMapping type, batch update database and invalidate cache
    pub async fn set_config<T: ConfigMapping>(&self, config: &T) -> Result<(), AppError> {
        log::debug!("🔧 Setting config from type: {}", std::any::type_name::<T>());
        
        // 1. Get field mappings
        let field_mappings = T::field_mappings();
        
        // 2. Serialize config to JSON to extract values
        let config_json = serde_json::to_value(config)
            .map_err(|e| AppError::unknown_with_params(
                "error.serialization",
                serde_json::json!({"msg": e.to_string()})
            ))?;
        
        let config_obj = config_json.as_object().ok_or_else(|| {
            AppError::unknown("error.invalid_config")
        })?;
        
        // 3. Batch update or insert configs
        let mut updated_count = 0;
        let mut inserted_count = 0;
        
        for field_mapping in field_mappings {
            let field_name = &field_mapping.field_name;
            let config_code = &field_mapping.config_code;
            
            // Convert field_name from snake_case to camelCase for JSON lookup
            // (e.g., "upload_service" -> "uploadService")
            let camel_case_name = to_camel_case(field_name);
            
            // Get value from serialized config
            // Try both camelCase (for #[serde(rename_all = "camelCase")]) and original field name
            let value = if let Some(field_value) = config_obj.get(&camel_case_name).or_else(|| config_obj.get(field_name)) {
                match field_value {
                    serde_json::Value::Null => {
                        // Skip null values, don't update/insert
                        log::debug!("⏭️  Skipping null value for field: {}", field_name);
                        continue;
                    }
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => field_value.to_string(),
                }
            } else {
                log::debug!("⏭️  Field {} (or {}) not found in config object", field_name, camel_case_name);
                continue;
            };
            
            // 4. Check if config exists
            let existing: Option<AppConfig> = self.rb
                .query_decode("SELECT * FROM app_config WHERE code = ?", vec![config_code.clone().into()])
                .await
                .map_err(|e| AppError::unknown_with_params(
                    "error.db_query",
                    serde_json::json!({"msg": e.to_string()})
                ))?;
            
            // 5. Update or insert
            if existing.is_some() {
                // Update existing config
                self.rb.exec(
                    "UPDATE app_config SET value = ?, update_time = NOW() WHERE code = ?",
                    vec![value.clone().into(), config_code.clone().into()]
                )
                .await
                .map_err(|e| AppError::unknown_with_params(
                    "error.db_update",
                    serde_json::json!({"msg": e.to_string()})
                ))?;
                
                updated_count += 1;
                log::debug!("✅ Config updated: {} = {}", config_code, value);
            } else {
                // Insert new config
                let new_config = AppConfig {
                    id: None,
                    code: Some(config_code.clone()),
                    value: Some(value.clone()),
                    remark: Some(format!("Auto-saved from {}", std::any::type_name::<T>())),
                    create_by: Some("system".to_string()),
                    create_time: Some(rbatis::rbdc::DateTime::now()),
                    update_by: None,
                    update_time: None,
                };
                
                AppConfig::insert(self.rb.as_ref(), &new_config)
                    .await
                    .map_err(|e| AppError::unknown_with_params(
                        "error.db_insert",
                        serde_json::json!({"msg": e.to_string()})
                    ))?;
                
                inserted_count += 1;
                log::debug!("✅ Config inserted: {} = {}", config_code, value);
            }
        }
        
        log::info!("📝 Config batch update completed: {} updated, {} inserted", updated_count, inserted_count);
        
        // 6. Invalidate cache
        self.invalidate_config_cache().await?;
        
        Ok(())
    }

    /// Get configuration value by code with caching and default value support
    pub async fn get_value_by_code(
        &self,
        code: &str,
        default_value: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        let cache_key = format!("app_config:single:{}", code);
        
        // 1. Try to get from Redis cache
        if let Ok(Some(cached_value)) = self.redis.get(&cache_key).await {
            log::debug!("📦 Config value loaded from cache: {} = {}", code, cached_value);
            return Ok(Some(cached_value));
        }
        
        log::debug!("🔍 Cache miss for {}, querying database", code);
        
        // 2. Query from database
        let config: Option<AppConfig> = self.rb
            .query_decode("SELECT * FROM app_config WHERE code = ?", vec![code.into()])
            .await
            .map_err(|e| AppError::unknown_with_params(
                "error.db_query",
                serde_json::json!({"msg": e.to_string()})
            ))?;
        
        if let Some(cfg) = config {
            // 3. Found in database, cache it
            if let Some(value) = cfg.value {
                if let Err(e) = self.redis.set_ex(&cache_key, &value, 3600).await {
                    log::warn!("Failed to cache config value: {}", e);
                } else {
                    log::debug!("💾 Config value cached: {} = {} (TTL: 3600s)", code, value);
                }
                return Ok(Some(value));
            }
        }
        
        // 4. Not found in database, use default value
        if let Some(default) = default_value {
            log::info!("⚙️  Config {} not found, using default value: {}", code, default);
            
            // 5. Save default value to database
            let new_config = AppConfig {
                id: None,
                code: Some(code.to_string()),
                value: Some(default.to_string()),
                remark: Some("Auto-generated from default value".to_string()),
                create_by: Some("system".to_string()),
                create_time: Some(rbatis::rbdc::DateTime::now()),
                update_by: None,
                update_time: None,
            };
            
            AppConfig::insert(self.rb.as_ref(), &new_config)
                .await
                .map_err(|e| AppError::unknown_with_params(
                    "error.db_insert",
                    serde_json::json!({"msg": e.to_string()})
                ))?;
            
            log::info!("✅ Default config saved to database: {} = {}", code, default);
            
            // 6. Cache the default value
            if let Err(e) = self.redis.set_ex(&cache_key, default, 3600).await {
                log::warn!("Failed to cache default config value: {}", e);
            } else {
                log::debug!("💾 Default config value cached: {} = {} (TTL: 3600s)", code, default);
            }
            
            return Ok(Some(default.to_string()));
        }
        
        // 7. No default value provided
        log::debug!("Config {} not found and no default value provided", code);
        Ok(None)
    }
}

/// Convert snake_case string to camelCase
/// 
/// # Examples
/// ```
/// assert_eq!(to_camel_case("upload_service"), "uploadService");
/// assert_eq!(to_camel_case("max_size_mb"), "maxSizeMb");
/// ```
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    
    result
}
