pub use serde::Deserialize;
pub use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String, // Unique key
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelType {
    pub name: String, // Unique key
    pub description: String,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub label_type: String,
    pub value: String, // TODO: more powerful generics?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInstance {
    pub config_name: String, // Unique key from Config
    pub instance: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigManSettings {
    pub version: String,
}
