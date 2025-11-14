//! HTTP route handlers for verification server

#[cfg(feature = "server")]
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::inference::InferenceAPI;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Verify request
#[derive(Deserialize)]
pub struct VerifyRequest {
    pub code: String,
    pub effect: String,
}

/// Infer request
#[derive(Deserialize)]
pub struct InferRequest {
    pub code: String,
}

/// Compose request
#[derive(Deserialize)]
pub struct ComposeRequest {
    pub words: Vec<String>,
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[cfg(feature = "server")]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(feature = "server")]
pub async fn verify(
    State(api): State<Arc<InferenceAPI>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<crate::inference::VerifyResult>, (StatusCode, Json<ErrorResponse>)> {
    match api.verify_effect(&req.code, &req.effect) {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[cfg(feature = "server")]
pub async fn infer(
    State(api): State<Arc<InferenceAPI>>,
    Json(req): Json<InferRequest>,
) -> Result<Json<crate::inference::InferenceResult>, (StatusCode, Json<ErrorResponse>)> {
    match api.infer(&req.code) {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[cfg(feature = "server")]
pub async fn compose(
    State(api): State<Arc<InferenceAPI>>,
    Json(req): Json<ComposeRequest>,
) -> Result<Json<crate::inference::CompositionResult>, (StatusCode, Json<ErrorResponse>)> {
    let words: Vec<&str> = req.words.iter().map(|s| s.as_str()).collect();
    match api.compose(&words) {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
    }
}
