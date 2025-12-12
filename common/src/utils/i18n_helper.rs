
use rust_i18n::t;
use serde_json::Value;
use std::collections::HashMap;

/// Replace placeholders in a template string with provided parameters
/// Supports {key} format placeholders
pub fn replace_params(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Convert JSON Value parameters to HashMap<String, String>
pub fn json_to_params(params: Value) -> HashMap<String, String> {
    let mut param_map: HashMap<String, String> = HashMap::new();
    
    if let Some(obj) = params.as_object() {
        for (k, v) in obj {
            let str_val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => v.to_string(),
            };
            param_map.insert(k.clone(), str_val);
        }
    }
    
    param_map
}

/// 使用 serde_json::Value 参数进行翻译
pub fn translate_with_params(key: &str, params: Option<Value>) -> String {
    if let Some(params_value) = params {
        let param_map = json_to_params(params_value);
        // rust_i18n 的 t! 宏需要在编译时使用，所以我们需要手动处理参数替换
        return replace_params(&t!(key).to_string(), &param_map);
    }

    t!(key).to_string()
}

/// 使用 HashMap 参数进行翻译
pub fn translate_with_hashmap<S: AsRef<str>>(key: &str, params: Option<HashMap<String, S>>) -> String {
    if let Some(param_map) = params {
        let str_map: HashMap<String, String> = param_map
            .iter()
            .map(|(k, v)| (k.clone(), v.as_ref().to_string()))
            .collect();
        return replace_params(&t!(key).to_string(), &str_map);
    }

    t!(key).to_string()
}

/// 便捷宏：构建 JSON 参数
#[macro_export]
macro_rules! i18n_params {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = serde_json::Map::new();
            $(
                map.insert($key.to_string(), serde_json::json!($value));
            )*
            Some(serde_json::Value::Object(map))
        }
    };
}