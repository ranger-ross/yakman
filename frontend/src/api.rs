use std::{collections::HashMap, error::Error};

use gloo_net::http::Request;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use yak_man_core::model::{
    Config, ConfigInstance, ConfigInstanceRevision, LabelType, OAuthExchangePayload,
    OAuthInitPayload,
};

use std::fmt;

pub async fn fetch_oauth_redirect_uri(
    challenge: PkceCodeChallenge,
) -> Result<String, RequestError> {
    let body = serde_json::to_string(&OAuthInitPayload {
        challenge: challenge,
    })?;
    return Ok(Request::post("/api/oauth2/init")
        .body(body)
        .header("content-type", "application/json")
        .send()
        .await?
        .text()
        .await?);
}

pub async fn exchange_oauth_code(
    code: &str,
    state: &str,
    verifier: PkceCodeVerifier,
) -> Result<String, RequestError> {
    let body = serde_json::to_string(&OAuthExchangePayload {
        code: String::from(code),
        state: String::from(state),
        verifier: verifier,
    })?;
    return Ok(Request::post("/api/oauth2/exchange")
        .body(body)
        .header("content-type", "application/json")
        .send()
        .await?
        .text()
        .await?);
}

pub async fn fetch_configs() -> Result<Vec<Config>, RequestError> {
    return Ok(Request::get("/api/configs").send().await?.json().await?);
}

pub async fn fetch_labels() -> Result<Vec<LabelType>, RequestError> {
    return Ok(Request::get("/api/labels").send().await?.json().await?);
}

pub async fn fetch_config_metadata(config_name: &str) -> Vec<ConfigInstance> {
    return Request::get(&format!("/api/configs/{config_name}/instances"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .expect("Failed to deserialize instance metadata JSON");
}

pub async fn fetch_instance_metadata(config_name: &str, instance: &str) -> ConfigInstance {
    return Request::get(&format!("/api/configs/{config_name}/instances/{instance}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .expect("Failed to deserialize instance metadata JSON");
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

    Request::put(&format!("/api/configs/{config_name}/instances"))
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
    content_type: Option<&str>,
) -> Result<(), RequestError> {
    let query_params: HashMap<&str, &str> = labels
        .iter()
        .map(|(key, value)| (&key[..], &value[..]))
        .collect();

    Request::post(&format!("/api/configs/{config_name}/instances/{instance}"))
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
        "/api/configs/{config_name}/instances/{instance}/revisions"
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
    Request::put(&format!(
        "/api/configs/{config_name}/instances/{instance}/revisions/{revision}/submit"
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
        "/api/configs/{config_name}/instances/{instance}/revisions/{revision}/approve"
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
