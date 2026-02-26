use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub type_: String,
    #[serde(rename = "unit")]
    pub unit: Option<String>,
    #[serde(rename = "nu_expr")]
    pub nu_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRange {
    pub rows: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    pub version: u32,
    pub columns: HashMap<String, ColumnMetadata>,
    pub named_ranges: HashMap<String, NamedRange>,
}

impl Sidecar {
    pub fn load_from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }

    pub fn get_column(&self, name: &str) -> Option<&ColumnMetadata> {
        self.columns.get(name)
    }

    pub fn is_derived_column(&self, name: &str) -> bool {
        if let Some(col) = self.columns.get(name) {
            col.type_ == "derived"
        } else {
            false
        }
    }
}
