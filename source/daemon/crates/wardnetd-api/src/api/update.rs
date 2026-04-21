//! `/api/update/*` — auto-update subsystem endpoints.
//!
//! All endpoints require admin authentication. Handlers are deliberately thin —
//! they call into [`UpdateService`](wardnetd_services::UpdateService), which
//! owns the state machine and background side effects.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use wardnet_common::api::{
    InstallUpdateRequest, InstallUpdateResponse, RollbackResponse, UpdateCheckResponse,
    UpdateConfigRequest, UpdateConfigResponse, UpdateHistoryResponse, UpdateStatusResponse,
};

use crate::api::middleware::AdminAuth;
use crate::api::responses::{AuthErrors, BadRequest};
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register auto-update routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(status))
        .routes(routes!(check))
        .routes(routes!(install))
        .routes(routes!(rollback))
        .routes(routes!(update_config))
        .routes(routes!(history))
}

/// Query parameters for GET /api/update/history.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct HistoryQuery {
    /// Max entries to return (default 20).
    #[serde(default = "default_history_limit")]
    pub limit: u32,
}

const fn default_history_limit() -> u32 {
    20
}

/// Get the current auto-update status.
#[utoipa::path(
    get,
    path = "/api/update/status",
    tag = "update",
    responses(
        (status = 200, description = "Current auto-update status", body = UpdateStatusResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn status(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<UpdateStatusResponse>, AppError> {
    Ok(Json(state.update_service().status().await?))
}

/// Force a manifest refresh to check for new releases.
#[utoipa::path(
    post,
    path = "/api/update/check",
    tag = "update",
    responses(
        (status = 200, description = "Manifest refresh result", body = UpdateCheckResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn check(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<UpdateCheckResponse>, AppError> {
    Ok(Json(state.update_service().check().await?))
}

/// Kick off an install of an available release.
#[utoipa::path(
    post,
    path = "/api/update/install",
    tag = "update",
    request_body(content = InstallUpdateRequest, description = "Install options; if omitted, installs the latest available release"),
    responses(
        (status = 200, description = "Install initiated", body = InstallUpdateResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn install(
    State(state): State<AppState>,
    _auth: AdminAuth,
    body: Option<Json<InstallUpdateRequest>>,
) -> Result<Json<InstallUpdateResponse>, AppError> {
    let req = body.map(|b| b.0).unwrap_or_default();
    Ok(Json(state.update_service().install(req).await?))
}

/// Roll back to the previous binary swapped aside as `<live>.old`.
#[utoipa::path(
    post,
    path = "/api/update/rollback",
    tag = "update",
    responses(
        (status = 200, description = "Rollback initiated", body = RollbackResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn rollback(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<RollbackResponse>, AppError> {
    Ok(Json(state.update_service().rollback().await?))
}

/// Toggle auto-update or switch release channel.
#[utoipa::path(
    put,
    path = "/api/update/config",
    tag = "update",
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Updated auto-update configuration", body = UpdateConfigResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn update_config(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<UpdateConfigRequest>,
) -> Result<Json<UpdateConfigResponse>, AppError> {
    Ok(Json(state.update_service().update_config(body).await?))
}

/// List recent install/rollback attempts.
#[utoipa::path(
    get,
    path = "/api/update/history",
    tag = "update",
    params(HistoryQuery),
    responses(
        (status = 200, description = "Update history", body = UpdateHistoryResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn history(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<UpdateHistoryResponse>, AppError> {
    Ok(Json(state.update_service().history(query.limit).await?))
}
