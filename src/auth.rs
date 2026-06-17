use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Semaphore;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;

use crate::config::{BackendDefinition, MulitAuthConfig};

pub struct Backend {
    pub def: BackendDefinition,
    pub semaphore: Semaphore,
}

pub struct ProxyState {
    pub client: reqwest::Client,
    pub backends: Vec<Backend>,
}

impl ProxyState {
    pub fn new(config: &MulitAuthConfig) -> Self {
        let backends = config
            .backends
            .iter()
            .map(|def| Backend {
                def: def.clone(),
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

pub enum RequestFailure {
    Reqwest(ReqwestError),
    ParseError { payload: String, err: SerdeError },
}

pub enum BackendStatus {
    ForbiddenByConfig,
    Invalid(Value),
    MissingIsValid(Value),
    InvalidJson { payload: String, err: String },
    ReqwestErr(ReqwestError),
}

impl BackendStatus {
    fn tag(&self) -> String {
        match self {
            Self::ForbiddenByConfig => "forbidden_uuid via config".to_string(),
            Self::Invalid(_) => "invalid".to_string(),
            Self::MissingIsValid(_) => "bad payload".to_string(),
            Self::InvalidJson { .. } => "invalid json".to_string(),
            Self::ReqwestErr(err) => {
                if err.is_timeout() {
                    "timed out".to_string()
                } else if let Some(status) = err.status() {
                    format!("http {}", status.as_u16())
                } else {
                    "network error".to_string()
                }
            }
        }
    }

    fn print_details(&self, idx: usize, url: &str) {
        match self {
            // no details necessary
            Self::ForbiddenByConfig => {}
            // an explicit isValid: false is normal, so we don't print anything
            Self::Invalid(_) => {}
            Self::MissingIsValid(json) => {
                println!("Backend {} ({}) missing 'isValid'. Payload: {}", idx, url, json);
            }
            Self::InvalidJson { payload, err } => {
                println!("Backend {} ({}) invalid JSON. Error: {}. Payload: {}", idx, url, err, payload);
            }
            Self::ReqwestErr(e) => {
                println!("Backend {} ({}) request failed: {}", idx, url, e);
            }
        }
    }
}

async fn auth_request(
    client: &reqwest::Client,
    backend_url: &str,
    semaphore: &tokio::sync::Semaphore,
    hash: &str,
    user_id: &str,
) -> Result<Value, RequestFailure> {
    let url = format!(
        "{}/api/session/hasJoined?hash={}&userId={}",
        backend_url, hash, user_id
    );

    let _ = semaphore.acquire().await.expect("Semaphore closed");

    let res = client.get(&url).send().await.map_err(RequestFailure::Reqwest)?;
    let raw_text = res.text().await.map_err(RequestFailure::Reqwest)?;

    serde_json::from_str(&raw_text).map_err(|err| RequestFailure::ParseError {
        payload: raw_text,
        err,
    })
}

pub async fn has_joined_handler(
    State(proxy_state): State<Arc<ProxyState>>,
    Query(params): Query<HasJoinedParams>,
) -> Result<Json<Value>, StatusCode> {
    let total = proxy_state.backends.len();
    let mut statuses: Vec<(usize, String, BackendStatus)> = Vec::with_capacity(total);
    /*
     I don't want to hardcode any assumptions about the payload structure besides isValid
     so if we fail on all backends we just forward the last auth result along to SS14
    */
    let mut last_response: Option<Value> = None;

    for (i, backend) in proxy_state.backends.iter().enumerate() {
        let backend_num = i + 1;

        if backend.def.forbidden_uuids.contains(&params.user_id) {
            statuses.push((backend_num, backend.def.url.clone(), BackendStatus::ForbiddenByConfig));
            continue;
        }

        let result = auth_request(
            &proxy_state.client,
            &backend.def.url,
            &backend.semaphore,
            &params.hash,
            &params.user_id,
        )
            .await;

        let status = match result {
            Ok(json) => {
                if let Some(is_valid) = json.get("isValid").and_then(|v| v.as_bool()) {
                    if is_valid {
                        println!(
                            "Authentication succeeded for {} on backend {}/{} ({})",
                            params.user_id, backend_num, total, backend.def.url
                        );
                        return Ok(Json(json));
                    }
                    last_response = Some(json.clone());
                    BackendStatus::Invalid(json)
                } else {
                    last_response = Some(json.clone());
                    BackendStatus::MissingIsValid(json)
                }
            }
            Err(RequestFailure::ParseError { payload, err }) => {
                BackendStatus::InvalidJson { payload, err: err.to_string() }
            }
            Err(RequestFailure::Reqwest(e)) => BackendStatus::ReqwestErr(e),
        };

        status.print_details(backend_num, &backend.def.url);

        statuses.push((backend_num, backend.def.url.clone(), status));
    }

    println!(
        "User {} failed authentication on all backends ({})",
        params.user_id,
        statuses
            .into_iter()
            .map(|(i, _, stat)| {
                format!("{}: {}", i, stat.tag())
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    match last_response {
        Some(data) => Ok(Json(data)),
        None => Err(StatusCode::BAD_GATEWAY),
    }
}