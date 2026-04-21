//! `/api/update/*` — auto-update subsystem endpoints.
//!
//! All endpoints require admin authentication. Handlers are deliberately thin —
//! they call into [`UpdateService`](wardnetd_services::UpdateService), which
//! owns the state machine and background side effects.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use wardnet_common::api::{
    InstallUpdateRequest, InstallUpdateResponse, RollbackResponse, UpdateCheckResponse,
    UpdateConfigRequest, UpdateConfigResponse, UpdateHistoryResponse, UpdateStatusResponse,
};

use crate::api::middleware::AdminAuth;
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Query parameters for GET /api/update/history.
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    /// Max entries to return (default 20).
    #[serde(default = "default_history_limit")]
    pub limit: u32,
}

const fn default_history_limit() -> u32 {
    20
}

/// GET /api/update/status
pub async fn status(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<UpdateStatusResponse>, AppError> {
    Ok(Json(state.update_service().status().await?))
}

/// POST /api/update/check — force a manifest refresh.
pub async fn check(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<UpdateCheckResponse>, AppError> {
    Ok(Json(state.update_service().check().await?))
}

/// POST /api/update/install — kick off an install.
pub async fn install(
    State(state): State<AppState>,
    _auth: AdminAuth,
    body: Option<Json<InstallUpdateRequest>>,
) -> Result<Json<InstallUpdateResponse>, AppError> {
    let req = body.map(|b| b.0).unwrap_or_default();
    Ok(Json(state.update_service().install(req).await?))
}

/// POST /api/update/rollback — swap back to `<live>.old`.
pub async fn rollback(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<RollbackResponse>, AppError> {
    Ok(Json(state.update_service().rollback().await?))
}

/// PUT /api/update/config — toggle auto-update / switch channel.
pub async fn update_config(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<UpdateConfigRequest>,
) -> Result<Json<UpdateConfigResponse>, AppError> {
    Ok(Json(state.update_service().update_config(body).await?))
}

/// GET /api/update/history?limit=N
pub async fn history(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<UpdateHistoryResponse>, AppError> {
    Ok(Json(state.update_service().history(query.limit).await?))
}
