use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use wardnet_common::api::{
    CreateAllowlistRequest, CreateAllowlistResponse, CreateBlocklistRequest,
    CreateBlocklistResponse, CreateFilterRuleRequest, CreateFilterRuleResponse,
    DeleteAllowlistResponse, DeleteBlocklistResponse, DeleteFilterRuleResponse,
    DnsCacheFlushResponse, DnsConfigResponse, DnsStatusResponse, ListAllowlistResponse,
    ListBlocklistsResponse, ListFilterRulesResponse, ToggleDnsRequest, UpdateBlocklistRequest,
    UpdateBlocklistResponse, UpdateDnsConfigRequest, UpdateFilterRuleRequest,
    UpdateFilterRuleResponse,
};
use wardnet_common::jobs::JobDispatchedResponse;

use crate::api::middleware::AdminAuth;
use crate::api::responses::{AuthErrors, BadRequest, NotFound};
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register DNS routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_config, update_config))
        .routes(routes!(toggle))
        .routes(routes!(status))
        .routes(routes!(flush_cache))
        .routes(routes!(list_blocklists, create_blocklist))
        .routes(routes!(update_blocklist, delete_blocklist))
        .routes(routes!(update_blocklist_now))
        .routes(routes!(list_allowlist, create_allowlist_entry))
        .routes(routes!(delete_allowlist_entry))
        .routes(routes!(list_filter_rules, create_filter_rule))
        .routes(routes!(update_filter_rule, delete_filter_rule))
}

/// Get the current DNS filter configuration.
#[utoipa::path(
    get,
    path = "/api/dns/config",
    tag = "dns",
    responses(
        (status = 200, description = "Current DNS configuration", body = DnsConfigResponse),
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
) -> Result<Json<DnsConfigResponse>, AppError> {
    let response = state.dns_service().get_config().await?;
    Ok(Json(response))
}

/// Update the DNS filter configuration.
#[utoipa::path(
    put,
    path = "/api/dns/config",
    tag = "dns",
    request_body = UpdateDnsConfigRequest,
    responses(
        (status = 200, description = "Updated DNS configuration", body = DnsConfigResponse),
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
    Json(body): Json<UpdateDnsConfigRequest>,
) -> Result<Json<DnsConfigResponse>, AppError> {
    let response = state.dns_service().update_config(body).await?;
    Ok(Json(response))
}

/// Enable or disable the DNS filter server.
#[utoipa::path(
    post,
    path = "/api/dns/config/toggle",
    tag = "dns",
    request_body = ToggleDnsRequest,
    responses(
        (status = 200, description = "Updated DNS configuration", body = DnsConfigResponse),
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
    Json(body): Json<ToggleDnsRequest>,
) -> Result<Json<DnsConfigResponse>, AppError> {
    let enabled = body.enabled;
    let response = state.dns_service().toggle(body).await?;

    if enabled {
        if let Err(e) = state.dns_server().start().await {
            tracing::error!(error = %e, "failed to start DNS server");
        }
    } else if let Err(e) = state.dns_server().stop().await {
        tracing::error!(error = %e, "failed to stop DNS server");
    }

    Ok(Json(response))
}

/// Get DNS server status and cache statistics.
#[utoipa::path(
    get,
    path = "/api/dns/status",
    tag = "dns",
    responses(
        (status = 200, description = "DNS server status and cache stats", body = DnsStatusResponse),
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
) -> Result<Json<DnsStatusResponse>, AppError> {
    let config = state.dns_service().get_dns_config().await?;
    let server = state.dns_server();
    Ok(Json(DnsStatusResponse {
        enabled: config.enabled,
        running: server.is_running(),
        cache_size: server.cache_size().await,
        cache_capacity: config.cache_size,
        cache_hit_rate: server.cache_hit_rate().await,
    }))
}

/// Flush the DNS resolver cache.
#[utoipa::path(
    post,
    path = "/api/dns/cache/flush",
    tag = "dns",
    responses(
        (status = 200, description = "Cache flushed", body = DnsCacheFlushResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn flush_cache(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<DnsCacheFlushResponse>, AppError> {
    let cleared = state.dns_server().flush_cache().await;
    Ok(Json(DnsCacheFlushResponse {
        message: "Cache flushed".to_owned(),
        entries_cleared: cleared,
    }))
}

// ---------------------------------------------------------------------------
// Blocklists
// ---------------------------------------------------------------------------

/// List all configured blocklists.
#[utoipa::path(
    get,
    path = "/api/dns/blocklists",
    tag = "dns",
    responses(
        (status = 200, description = "List of blocklists", body = ListBlocklistsResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_blocklists(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListBlocklistsResponse>, AppError> {
    let response = state.dns_service().list_blocklists().await?;
    Ok(Json(response))
}

/// Create a new blocklist source.
#[utoipa::path(
    post,
    path = "/api/dns/blocklists",
    tag = "dns",
    request_body = CreateBlocklistRequest,
    responses(
        (status = 201, description = "Blocklist created", body = CreateBlocklistResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn create_blocklist(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<CreateBlocklistRequest>,
) -> Result<(StatusCode, Json<CreateBlocklistResponse>), AppError> {
    let response = state.dns_service().create_blocklist(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Update an existing blocklist by ID.
#[utoipa::path(
    put,
    path = "/api/dns/blocklists/{id}",
    tag = "dns",
    params(("id" = Uuid, Path, description = "Blocklist ID")),
    request_body = UpdateBlocklistRequest,
    responses(
        (status = 200, description = "Updated blocklist", body = UpdateBlocklistResponse),
        AuthErrors,
        NotFound,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn update_blocklist(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBlocklistRequest>,
) -> Result<Json<UpdateBlocklistResponse>, AppError> {
    let response = state.dns_service().update_blocklist(id, body).await?;
    Ok(Json(response))
}

/// Delete a blocklist by ID.
#[utoipa::path(
    delete,
    path = "/api/dns/blocklists/{id}",
    tag = "dns",
    params(("id" = Uuid, Path, description = "Blocklist ID")),
    responses(
        (status = 200, description = "Blocklist deleted", body = DeleteBlocklistResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn delete_blocklist(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteBlocklistResponse>, AppError> {
    let response = state.dns_service().delete_blocklist(id).await?;
    Ok(Json(response))
}

/// Trigger an immediate refresh of a blocklist.
///
/// Dispatches a background job that fetches, parses, and stores the blocklist.
/// Returns 202 Accepted with the job id so the client can poll the jobs
/// endpoint for progress.
#[utoipa::path(
    post,
    path = "/api/dns/blocklists/{id}/update",
    tag = "dns",
    params(("id" = Uuid, Path, description = "Blocklist ID")),
    responses(
        (status = 202, description = "Background update job dispatched", body = JobDispatchedResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn update_blocklist_now(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<JobDispatchedResponse>), AppError> {
    let response = state.dns_service().update_blocklist_now(id).await?;
    Ok((StatusCode::ACCEPTED, Json(response)))
}

// ---------------------------------------------------------------------------
// Allowlist
// ---------------------------------------------------------------------------

/// List all DNS allowlist entries.
#[utoipa::path(
    get,
    path = "/api/dns/allowlist",
    tag = "dns",
    responses(
        (status = 200, description = "List of allowlist entries", body = ListAllowlistResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_allowlist(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListAllowlistResponse>, AppError> {
    let response = state.dns_service().list_allowlist().await?;
    Ok(Json(response))
}

/// Add a domain to the DNS allowlist.
#[utoipa::path(
    post,
    path = "/api/dns/allowlist",
    tag = "dns",
    request_body = CreateAllowlistRequest,
    responses(
        (status = 201, description = "Allowlist entry created", body = CreateAllowlistResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn create_allowlist_entry(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<CreateAllowlistRequest>,
) -> Result<(StatusCode, Json<CreateAllowlistResponse>), AppError> {
    let response = state.dns_service().create_allowlist_entry(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Remove a domain from the DNS allowlist.
#[utoipa::path(
    delete,
    path = "/api/dns/allowlist/{id}",
    tag = "dns",
    params(("id" = Uuid, Path, description = "Allowlist entry ID")),
    responses(
        (status = 200, description = "Allowlist entry deleted", body = DeleteAllowlistResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn delete_allowlist_entry(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteAllowlistResponse>, AppError> {
    let response = state.dns_service().delete_allowlist_entry(id).await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// Custom filter rules
// ---------------------------------------------------------------------------

/// List all custom DNS filter rules.
#[utoipa::path(
    get,
    path = "/api/dns/rules",
    tag = "dns",
    responses(
        (status = 200, description = "List of custom filter rules", body = ListFilterRulesResponse),
        AuthErrors,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn list_filter_rules(
    State(state): State<AppState>,
    _auth: AdminAuth,
) -> Result<Json<ListFilterRulesResponse>, AppError> {
    let response = state.dns_service().list_filter_rules().await?;
    Ok(Json(response))
}

/// Create a custom DNS filter rule.
#[utoipa::path(
    post,
    path = "/api/dns/rules",
    tag = "dns",
    request_body = CreateFilterRuleRequest,
    responses(
        (status = 201, description = "Filter rule created", body = CreateFilterRuleResponse),
        AuthErrors,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn create_filter_rule(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Json(body): Json<CreateFilterRuleRequest>,
) -> Result<(StatusCode, Json<CreateFilterRuleResponse>), AppError> {
    let response = state.dns_service().create_filter_rule(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Update a custom DNS filter rule by ID.
#[utoipa::path(
    put,
    path = "/api/dns/rules/{id}",
    tag = "dns",
    params(("id" = Uuid, Path, description = "Filter rule ID")),
    request_body = UpdateFilterRuleRequest,
    responses(
        (status = 200, description = "Updated filter rule", body = UpdateFilterRuleResponse),
        AuthErrors,
        NotFound,
        BadRequest,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn update_filter_rule(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFilterRuleRequest>,
) -> Result<Json<UpdateFilterRuleResponse>, AppError> {
    let response = state.dns_service().update_filter_rule(id, body).await?;
    Ok(Json(response))
}

/// Delete a custom DNS filter rule by ID.
#[utoipa::path(
    delete,
    path = "/api/dns/rules/{id}",
    tag = "dns",
    params(("id" = Uuid, Path, description = "Filter rule ID")),
    responses(
        (status = 200, description = "Filter rule deleted", body = DeleteFilterRuleResponse),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn delete_filter_rule(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteFilterRuleResponse>, AppError> {
    let response = state.dns_service().delete_filter_rule(id).await?;
    Ok(Json(response))
}
