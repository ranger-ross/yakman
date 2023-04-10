use gloo_net::http::Request;
use yak_man_core::model::{Config, ConfigInstance, Label, LabelType};

pub async fn fetch_configs() -> Vec<Config> {
    return Request::get("/api/configs")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

pub async fn fetch_labels() -> Vec<LabelType> {
    return Request::get("/api/labels")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

pub async fn fetch_instance_metadata(config_name: &str) -> Vec<ConfigInstance> {
    return Request::get(&format!("/api/instances/{config_name}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

pub async fn create_config_instance(config_name: &str, data: &str) {
    Request::put(&format!("/api/config/{config_name}/data"))
        .body(data)
        .send()
        .await
        .unwrap();
}

pub async fn update_config_instance(config_name: &str, instance: &str, data: &str) {
    Request::post(&format!("/api/config/{config_name}/instance/{instance}"))
        .body(data)
        .send()
        .await
        .unwrap();
}

pub async fn create_config(config_name: &str) {
    Request::put(&format!("/api/config/{config_name}"))
        .send()
        .await
        .unwrap();
}

pub async fn create_label(label: LabelType) {
    let body = serde_json::to_string(&label).unwrap();
    Request::put(&format!("/api/labels"))
        .body(body)
        .send()
        .await
        .unwrap();
}
