use std::{collections::HashMap, error::Error};

use gloo_net::http::Request;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, LabelType};

use std::fmt;

pub async fn fetch_configs() -> Result<Vec<Config>, RequestError> {
    return Ok(Request::get("/api/configs").send().await?.json().await?);
}

pub async fn fetch_labels() -> Result<Vec<LabelType>, RequestError> {
    return Ok(Request::get("/api/labels").send().await?.json().await?);
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

pub async fn create_config_instance(
    config_name: &str,
    data: &str,
    labels: HashMap<String, String>,
    content_type: Option<&str>,
) -> Result<(), RequestError> {
    let query_params: HashMap<&str, &str> = labels
        .iter()
        .map(|(key, value)| (&key[..], &value[..]))
        .collect();

    Request::put(&format!("/api/config/{config_name}/data"))
        .query(query_params)
        .header("content-type", content_type.unwrap_or("text/plain"))
        .body(data)
        .send()
        .await?;
    return Ok(());
}

pub async fn update_config_instance(
    config_name: &str,
    instance: &str,
    data: &str,
    labels: HashMap<String, String>,
    content_type: Option<&str>
) -> Result<(), RequestError> {
    let query_params: HashMap<&str, &str> = labels
        .iter()
        .map(|(key, value)| (&key[..], &value[..]))
        .collect();

    Request::post(&format!("/api/config/{config_name}/instance/{instance}"))
        .query(query_params)
        .header("content-type", content_type.unwrap_or("text/plain"))
        .body(data)
        .send()
        .await?;
    return Ok(());
}

pub async fn create_config(config_name: &str) -> Result<(), RequestError> {
    Request::put(&format!("/api/configs/{config_name}"))
        .send()
        .await?;
    return Ok(());
}

pub async fn create_label(label: LabelType) -> Result<(), RequestError> {
    let body = serde_json::to_string(&label)?;
    Request::put("/api/labels")
        .body(body)
        .header("content-type", "application/json")
        .send()
        .await?;
    return Ok(());
}

pub async fn fetch_instance_revisions(
    config_name: &str,
    instance: &str,
) -> Result<Vec<ConfigInstanceRevision>, RequestError> {
    return Ok(Request::get(&format!(
        "/api/config/{config_name}/instance/{instance}/revisions"
    ))
    .send()
    .await?
    .json()
    .await?);
}

pub async fn update_instance_revision(
    config_name: &str,
    instance: &str,
    revision: &str,
) -> Result<(), RequestError> {
    Request::post(&format!(
        "/api/config/{config_name}/instance/{instance}/revision/{revision}/current"
    ))
    .send()
    .await?;

    return Ok(());
}

pub async fn approve_instance_revision(
    config_name: &str,
    instance: &str,
    revision: &str,
) -> Result<(), RequestError> {
    Request::post(&format!(
        "/api/config/{config_name}/instance/{instance}/revision/{revision}/approve"
    ))
    .send()
    .await?;

    return Ok(());
}

#[derive(Debug)]
pub enum RequestError {
    Reqwest(gloo_net::Error),
    Json(serde_json::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::Reqwest(ref e) => e.fmt(f),
            RequestError::Json(ref e) => e.fmt(f),
        }
    }
}

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            RequestError::Reqwest(ref e) => Some(e),
            RequestError::Json(ref e) => Some(e),
        }
    }
}

impl From<gloo_net::Error> for RequestError {
    fn from(err: gloo_net::Error) -> RequestError {
        RequestError::Reqwest(err)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(err: serde_json::Error) -> RequestError {
        RequestError::Json(err)
    }
}
