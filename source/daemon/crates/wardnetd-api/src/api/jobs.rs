use axum::Json;
use axum::extract::{Path, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use wardnet_common::jobs::Job;

use crate::api::middleware::AdminAuth;
use crate::api::responses::{AuthErrors, NotFound};
use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register job routes onto the given [`OpenApiRouter`].
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(get_job))
}

/// GET /api/jobs/{id} — poll the status of a background job. Returns 404 when
/// the job id is unknown (either never dispatched or GC'd after its TTL).
#[utoipa::path(
    get,
    path = "/api/jobs/{id}",
    tag = "jobs",
    params(("id" = Uuid, Path, description = "Job ID")),
    responses(
        (status = 200, description = "Current job state", body = Job),
        AuthErrors,
        NotFound,
    ),
    security(
        ("session_cookie" = []),
        ("bearer_auth" = []),
    ),
)]
pub async fn get_job(
    State(state): State<AppState>,
    _auth: AdminAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, AppError> {
    state
        .job_service()
        .get(id)
        .await
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("job {id} not found")))
}
