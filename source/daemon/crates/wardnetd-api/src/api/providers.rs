use axum::Json;
use axum::extract::{Path, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use wardnet_common::api::{
    ListCountriesResponse, ListProvidersResponse, ListServersRequest, ListServersResponse,
    SetupProviderRequest, SetupProviderResponse, ValidateCredentialsRequest,
    ValidateCredentialsResponse,
};

use crate::api::middleware::AdminAuth;
use crate::api::responses::{AuthErrors, BadRequest, NotFound};
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register VPN provider routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(list_providers))
        .routes(routes!(validate_credentials))
        .routes(routes!(list_countries))
        .routes(routes!(list_servers))
        .routes(routes!(setup_tunnel))
}

/// List all registered VPN providers.
#[utoipa::path(
    get,
    path = "/api/providers",
    tag = "providers",
    responses(
        (status = 200, description = "Registered VPN providers", body = ListProvidersResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_providers(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListProvidersResponse>, AppError> {
    let response = state.provider_service().list_providers().await?;
    Ok(Json(response))
}

/// Validate credentials against a VPN provider.
#[utoipa::path(
    post,
    path = "/api/providers/{id}/validate",
    tag = "providers",
    params(("id" = String, Path, description = "Provider ID")),
    request_body = ValidateCredentialsRequest,
    responses(
        (status = 200, description = "Validation result", body = ValidateCredentialsResponse),
        AuthErrors,
        NotFound,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn validate_credentials(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(body): Json<ValidateCredentialsRequest>,
) -> Result<Json<ValidateCredentialsResponse>, AppError> {
    let response = state
        .provider_service()
        .validate_credentials(&id, body)
        .await?;
    Ok(Json(response))
}

/// List countries where a VPN provider has servers.
#[utoipa::path(
    get,
    path = "/api/providers/{id}/countries",
    tag = "providers",
    params(("id" = String, Path, description = "Provider ID")),
    responses(
        (status = 200, description = "Countries with available servers", body = ListCountriesResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_countries(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<ListCountriesResponse>, AppError> {
    let response = state.provider_service().list_countries(&id).await?;
    Ok(Json(response))
}

/// List available VPN servers for a provider using supplied credentials.
#[utoipa::path(
    post,
    path = "/api/providers/{id}/servers",
    tag = "providers",
    params(("id" = String, Path, description = "Provider ID")),
    request_body = ListServersRequest,
    responses(
        (status = 200, description = "Available servers", body = ListServersResponse),
        AuthErrors,
        NotFound,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_servers(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(body): Json<ListServersRequest>,
) -> Result<Json<ListServersResponse>, AppError> {
    let response = state.provider_service().list_servers(&id, body).await?;
    Ok(Json(response))
}

/// Run full guided tunnel setup through a VPN provider.
#[utoipa::path(
    post,
    path = "/api/providers/{id}/setup",
    tag = "providers",
    params(("id" = String, Path, description = "Provider ID")),
    request_body = SetupProviderRequest,
    responses(
        (status = 200, description = "Guided tunnel setup result", body = SetupProviderResponse),
        AuthErrors,
        NotFound,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn setup_tunnel(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(body): Json<SetupProviderRequest>,
) -> Result<Json<SetupProviderResponse>, AppError> {
    let response = state.provider_service().setup_tunnel(&id, body).await?;
    Ok(Json(response))
}
