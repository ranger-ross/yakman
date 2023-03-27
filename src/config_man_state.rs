pub use serde::Deserialize;
pub use serde::Serialize;

use crate::data_types::{Config, LabelType};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigManState {
    pub configs: Vec<Config>,
    pub labels: Vec<LabelType>,
}
