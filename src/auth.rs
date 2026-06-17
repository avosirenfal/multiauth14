use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::config::FaitConfig;

pub struct Backend {
    pub url: String,
    pub semaphore: Semaphore,
}

pub struct ProxyState {
    pub client: reqwest::Client,
    pub backends: Vec<Backend>,
}

impl ProxyState {
    pub fn new(config: &FaitConfig) -> Self {
        let backends = config
            .backends
            .iter()
            .map(|url| Backend {
                url: url.clone(),
                semaphore: Semaphore::new(5),
            })
            .collect();

        Self {
            client: reqwest::Client::new(),
            backends,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HasJoinedParams {
    pub hash: String,
    pub user_id: String,
}

async fn auth_request(
    client: &reqwest::Client,
    backend_url: &str,
    semaphore: &Semaphore,
    hash: &str,
    user_id: &str,
) -> Result<Value, ()> {
    let url = format!(
        "{}/api/session/hasJoined?hash={}&userId={}",
        backend_url,
        hash,
        user_id
    );

    println!("Doing request: {}", url);

    let _ = semaphore.acquire().await.expect("Semaphore closed");


    let res = client.get(&url).send().await.map_err(|e| {
        eprintln!("Failed to connect to {}: {}", backend_url, e);
    })?;

    let raw_text = res.text().await.map_err(|e| {
        eprintln!("Failed to read response text from {}: {}", backend_url, e);
    })?;

    println!("Raw payload: {}", raw_text);

    let json: Value = serde_json::from_str(&raw_text).map_err(|e| {
        eprintln!("Failed to parse JSON from {}: {}", backend_url, e);
    })?;

    // let res = client.get(&url).send().await.map_err(|e| {
    //     eprintln!("Failed to connect to {}: {}", backend_url, e);
    // })?;
    //
    // let json: Value = res.json().await.map_err(|e| {
    //     eprintln!("Failed to parse JSON from {}: {}", backend_url, e);
    // })?;

    Ok(json)
}

pub async fn has_joined_handler(
    State(proxy_state): State<Arc<ProxyState>>,
    Query(params): Query<HasJoinedParams>,
) -> Result<Json<Value>, StatusCode> {

    let mut last_response: Option<Value> = None;

    for backend in &proxy_state.backends {
        if let Ok(json) = auth_request(&proxy_state.client, &backend.url, &backend.semaphore, &params.hash, &params.user_id).await {

            let is_valid = json.get("isValid")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if is_valid {
                println!("Authentication succeeded on backend {}", backend.url);
                return Ok(Json(json));
            } else {
                println!("Authentication failed on backend {}", backend.url);
            }

            last_response = Some(json);
        }
    }

    match last_response {
        Some(data) => Ok(Json(data)),
        None => Err(StatusCode::BAD_GATEWAY),
    }
}