pub mod auth;
pub mod config;

use std::error::Error;
use axum::{routing::get, Router};
use std::sync::Arc;

use crate::auth::{has_joined_handler, ProxyState};
use crate::config::MulitAuthConfig;

pub async fn start(state: Arc<ProxyState>, hostname: &str, port: u16) {
    let app = Router::new()
        .route("/api/session/hasJoined", get(has_joined_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((hostname, port))
        .await
        .expect("Failed to bind to provided hostname and port.");

    let addr = listener.local_addr().expect("Failed to get local address.");
    println!("Proxy listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config: MulitAuthConfig = confy::load_path("config.yaml")?;
    let proxy_state = Arc::new(ProxyState::new(&config));
    start(proxy_state, &config.host, config.port).await;
    Ok(())
}