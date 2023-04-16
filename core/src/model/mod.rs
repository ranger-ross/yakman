pub use serde::Deserialize;
pub use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub name: String, // Unique key
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LabelType {
    pub name: String, // Unique key
    pub description: String,
    pub priority: i32,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Label {
    pub label_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ConfigInstance {
    pub config_name: String,
    pub instance: String, // Unique key
    pub labels: Vec<Label>, // These should match the labels in the current revision
    pub current_revision: String,
    pub pending_revision: Option<String>,
    pub revisions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ConfigInstanceRevision {
    pub revision: String, // Unique key
    pub data_key: String, // Key to fetch data
    pub labels: Vec<Label>,
    pub timestamp_ms: i64,
    pub approved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YakManSettings {
    pub version: String,
}
