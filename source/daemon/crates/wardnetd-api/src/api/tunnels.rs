use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use wardnet_common::api::{
    CreateTunnelRequest, CreateTunnelResponse, DeleteTunnelResponse, ListTunnelsResponse,
};

use crate::api::middleware::AdminAuth;
use crate::api::responses::{AuthErrors, BadRequest, NotFound};
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register tunnel routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(list_tunnels, create_tunnel))
        .routes(routes!(delete_tunnel))
}

/// List all configured VPN tunnels.
///
/// Thin handler — lists all configured tunnels. Requires admin authentication.
#[utoipa::path(
    get,
    path = "/api/tunnels",
    tag = "tunnels",
    responses(
        (status = 200, description = "Configured tunnels", body = ListTunnelsResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_tunnels(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListTunnelsResponse>, AppError> {
    let response = state.tunnel_service().list_tunnels().await?;
    Ok(Json(response))
}

/// Import a tunnel from a `WireGuard` `.conf` file.
///
/// Thin handler — imports a tunnel from a `WireGuard` `.conf` file.
/// Requires admin authentication.
#[utoipa::path(
    post,
    path = "/api/tunnels",
    tag = "tunnels",
    request_body = CreateTunnelRequest,
    responses(
        (status = 201, description = "Tunnel imported", body = CreateTunnelResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn create_tunnel(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<CreateTunnelRequest>,
) -> Result<(StatusCode, Json<CreateTunnelResponse>), AppError> {
    let response = state.tunnel_service().import_tunnel(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Delete a tunnel and its configuration by ID.
///
/// Thin handler — deletes a tunnel and its configuration.
/// Requires admin authentication.
#[utoipa::path(
    delete,
    path = "/api/tunnels/{id}",
    tag = "tunnels",
    params(("id" = Uuid, Path, description = "Tunnel ID")),
    responses(
        (status = 200, description = "Tunnel deleted", body = DeleteTunnelResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn delete_tunnel(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteTunnelResponse>, AppError> {
    let response = state.tunnel_service().delete_tunnel(id).await?;
    Ok(Json(response))
}
