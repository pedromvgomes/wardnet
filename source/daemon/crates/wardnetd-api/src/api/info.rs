use axum::Json;
use axum::extract::State;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use wardnet_common::api::{ApiError, InfoResponse};

use crate::state::AppState;

/// Register info routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(info))
}

/// GET /api/info
///
/// Thin handler — returns the daemon version and uptime.
/// No authentication required. Used by the web UI connection status widget.
#[utoipa::path(
    get,
    path = "/api/info",
    tag = "info",
    responses(
        (status = 200, description = "Daemon version and uptime", body = InfoResponse),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    security(()),
)]
pub async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    let uptime = state.system_service().uptime();
    Json(InfoResponse {
        version: state.system_service().version().to_owned(),
        uptime_seconds: uptime.as_secs(),
    })
}
