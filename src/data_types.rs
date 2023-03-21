pub use serde::Serialize;

#[derive(Serialize)]
pub struct AppConfig {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct AppLabelType {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct AppLabel {
    pub label_type: AppLabelType,
    pub value: String, // TODO: more powerful generics?
}

#[derive(Serialize)]
pub struct AppConfigInstance {
    pub config: AppConfig,
    pub content: String,
    pub labels: Vec<AppLabel>,
}
