use crate::model::{
    ConfigInstance, ConfigInstanceRevision, LabelType, YakManApiKey, YakManConfig, YakManUser,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelJson {
    pub labels: Vec<LabelType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigJson {
    pub configs: Vec<YakManConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceJson {
    pub instances: Vec<ConfigInstance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevisionJson {
    pub revision: ConfigInstanceRevision,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersJson {
    pub users: Vec<YakManUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeysJson {
    pub api_keys: Vec<YakManApiKey>,
}
