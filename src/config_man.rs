use crate::data_types::ConfigManSettings;

pub fn load_config_man_settings() -> ConfigManSettings {
    return ConfigManSettings {
        version: "0.0.1".to_string(),
    };
}
