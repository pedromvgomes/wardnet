use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use wardnet_common::api::{
    CreateDhcpReservationRequest, CreateDhcpReservationResponse, DeleteDhcpReservationResponse,
    DhcpConfigResponse, DhcpStatusResponse, ListDhcpLeasesResponse, ListDhcpReservationsResponse,
    RevokeDhcpLeaseResponse, ToggleDhcpRequest, UpdateDhcpConfigRequest,
};

use crate::api::middleware::AdminAuth;
use crate::api::responses::{AuthErrors, BadRequest, NotFound};
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register DHCP routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_config, update_config))
        .routes(routes!(toggle))
        .routes(routes!(list_leases))
        .routes(routes!(revoke_lease))
        .routes(routes!(list_reservations, create_reservation))
        .routes(routes!(delete_reservation))
        .routes(routes!(status))
}

const TAG: &str = "dhcp";
const PATH_CONFIG: &str = "/api/dhcp/config";
const PATH_TOGGLE: &str = "/api/dhcp/config/toggle";
const PATH_LEASES: &str = "/api/dhcp/leases";
const PATH_LEASE_ITEM: &str = "/api/dhcp/leases/{id}";
const PATH_RESERVATIONS: &str = "/api/dhcp/reservations";
const PATH_RESERVATION_ITEM: &str = "/api/dhcp/reservations/{id}";
const PATH_STATUS: &str = "/api/dhcp/status";

/// Get the current DHCP pool configuration.
///
/// Thin handler — returns the current DHCP pool configuration.
/// Requires admin authentication.
#[utoipa::path(
    get,
    path = PATH_CONFIG,
    tag = TAG,
    responses(
        (status = 200, description = "Current DHCP configuration", body = DhcpConfigResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn get_config(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<DhcpConfigResponse>, AppError> {
    let response = state.dhcp_service().get_config().await?;
    Ok(Json(response))
}

/// Update the DHCP pool configuration.
///
/// Thin handler — updates the DHCP pool configuration.
/// Requires admin authentication.
#[utoipa::path(
    put,
    path = PATH_CONFIG,
    tag = TAG,
    request_body = UpdateDhcpConfigRequest,
    responses(
        (status = 200, description = "Updated DHCP configuration", body = DhcpConfigResponse),
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
    Json(body): Json<UpdateDhcpConfigRequest>,
) -> Result<Json<DhcpConfigResponse>, AppError> {
    let response = state.dhcp_service().update_config(body).await?;
    Ok(Json(response))
}

/// Enable or disable the DHCP server.
///
/// Thin handler — enables or disables the DHCP server.
/// Requires admin authentication.
#[utoipa::path(
    post,
    path = PATH_TOGGLE,
    tag = TAG,
    request_body = ToggleDhcpRequest,
    responses(
        (status = 200, description = "Updated DHCP configuration", body = DhcpConfigResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn toggle(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<ToggleDhcpRequest>,
) -> Result<Json<DhcpConfigResponse>, AppError> {
    let enabled = body.enabled;
    let response = state.dhcp_service().toggle(body).await?;

    // Start or stop the DHCP server based on the new config.
    if enabled {
        state.dhcp_server().start().await?;
    } else {
        state.dhcp_server().stop().await?;
    }

    Ok(Json(response))
}

/// List all active DHCP leases.
///
/// Thin handler — lists all active DHCP leases.
/// Requires admin authentication.
#[utoipa::path(
    get,
    path = PATH_LEASES,
    tag = TAG,
    responses(
        (status = 200, description = "Active DHCP leases", body = ListDhcpLeasesResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_leases(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListDhcpLeasesResponse>, AppError> {
    let response = state.dhcp_service().list_leases().await?;
    Ok(Json(response))
}

/// Revoke an active DHCP lease by ID.
///
/// Thin handler — revokes an active DHCP lease.
/// Requires admin authentication.
#[utoipa::path(
    delete,
    path = PATH_LEASE_ITEM,
    tag = TAG,
    params(("id" = Uuid, Path, description = "Lease ID")),
    responses(
        (status = 200, description = "Lease revoked", body = RevokeDhcpLeaseResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn revoke_lease(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<RevokeDhcpLeaseResponse>, AppError> {
    let response = state.dhcp_service().revoke_lease(id).await?;
    Ok(Json(response))
}

/// List all static DHCP reservations.
///
/// Thin handler — lists all static DHCP reservations.
/// Requires admin authentication.
#[utoipa::path(
    get,
    path = PATH_RESERVATIONS,
    tag = TAG,
    responses(
        (status = 200, description = "Static DHCP reservations", body = ListDhcpReservationsResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_reservations(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListDhcpReservationsResponse>, AppError> {
    let response = state.dhcp_service().list_reservations().await?;
    Ok(Json(response))
}

/// Create a static MAC-to-IP DHCP reservation.
///
/// Thin handler — creates a new static MAC-to-IP reservation.
/// Requires admin authentication.
#[utoipa::path(
    post,
    path = PATH_RESERVATIONS,
    tag = TAG,
    request_body = CreateDhcpReservationRequest,
    responses(
        (status = 201, description = "Reservation created", body = CreateDhcpReservationResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn create_reservation(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<CreateDhcpReservationRequest>,
) -> Result<(StatusCode, Json<CreateDhcpReservationResponse>), AppError> {
    let response = state.dhcp_service().create_reservation(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Delete a static DHCP reservation by ID.
///
/// Thin handler — deletes a static DHCP reservation.
/// Requires admin authentication.
#[utoipa::path(
    delete,
    path = PATH_RESERVATION_ITEM,
    tag = TAG,
    params(("id" = Uuid, Path, description = "Reservation ID")),
    responses(
        (status = 200, description = "Reservation deleted", body = DeleteDhcpReservationResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn delete_reservation(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteDhcpReservationResponse>, AppError> {
    let response = state.dhcp_service().delete_reservation(id).await?;
    Ok(Json(response))
}

/// Get DHCP server status and pool usage.
///
/// Thin handler — returns DHCP server status and pool usage.
/// Requires admin authentication.
#[utoipa::path(
    get,
    path = PATH_STATUS,
    tag = TAG,
    responses(
        (status = 200, description = "DHCP server status and pool usage", body = DhcpStatusResponse),
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
) -> Result<Json<DhcpStatusResponse>, AppError> {
    let response = state.dhcp_service().status().await?;
    Ok(Json(response))
}
