use model::YakManSettings;

pub mod model;

pub fn load_yak_man_settings() -> YakManSettings {
    return YakManSettings {
        version: "0.0.1".to_string(),
    };
}
