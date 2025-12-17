use serde::{Deserialize, Serialize};

/// 标签（用于下拉选项）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label<V, L> {
    pub value: V,
    pub label: L,
}