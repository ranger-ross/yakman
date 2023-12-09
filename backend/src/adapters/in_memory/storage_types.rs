use serde::{Serialize, Deserialize};
use crate::model::{LabelType, YakManConfig, ConfigInstance, ConfigInstanceRevision, YakManUser};

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

