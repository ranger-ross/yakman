pub use serde::Deserialize;
pub use serde::Serialize;

use crate::data_types::{AppConfig, AppLabelType};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigManState {
    pub configs: Vec<AppConfig>,
    pub labels: Vec<AppLabelType>,
}
