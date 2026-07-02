//! vLLM Bridge Sidecar
//!
//! Provides a Weft-compatible HTTP interface for vLLM inference.
//! Implements the three required sidecar endpoints:
//! - POST /action  - accepts { action, payload }, returns { result }
//! - GET /health   - liveness check
//! - GET /outputs  - runtime-computed values for node output ports

use axum::{Router, routing::{get, post}, Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    vllm_url: String,
}

#[derive(Debug, Deserialize)]
struct ActionRequest {
    action: String,
    payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ActionResponse {
    result: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct OutputsResponse {
    endpoint_url: String,
    vllm_url: String,
}

async fn health() -> &'static str {
    "ok"
}

async fn outputs(State(state): State<Arc<AppState>>) -> Json<OutputsResponse> {
    Json(OutputsResponse {
        endpoint_url: "http://localhost:8081".to_string(),
        vllm_url: state.vllm_url.clone(),
    })
}

async fn action(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ActionRequest>,
) -> Json<ActionResponse> {
    match req.action.as_str() {
        "infer" => {
            let client = reqwest::Client::new();
            let url = format!("{}/v1/chat/completions", state.vllm_url);

            match client.post(&url).json(&req.payload).send().await {
                Ok(response) => {
                    let data: serde_json::Value = response.json().await.unwrap_or_default();
                    Json(ActionResponse { result: data })
                }
                Err(e) => {
                    Json(ActionResponse {
                        result: serde_json::json!({ "error": e.to_string() }),
                    })
                }
            }
        }
        _ => {
            Json(ActionResponse {
                result: serde_json::json!({ "error": format!("Unknown action: {}", req.action) }),
            })
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let vllm_url = std::env::var("VLLM_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());

    let state = Arc::new(AppState { vllm_url: vllm_url.clone() });

    let app = Router::new()
        .route("/health", get(health))
        .route("/outputs", get(outputs))
        .route("/action", post(action))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    tracing::info!("vLLM Bridge listening on :8081, vLLM backend: {}", vllm_url);
    axum::serve(listener, app).await.unwrap();
}
