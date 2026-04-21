use axum::Json;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use wardnet_common::api::{ApiError, LoginRequest, LoginResponse};

use crate::state::AppState;
use wardnetd_services::error::AppError;

/// Register auth routes onto the given [`OpenApiRouter`]. Each module owns its
/// own route list so `api::mod::router` stays a simple composition point.
pub fn register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(login))
}

/// Log in with username and password, issuing a session cookie.
///
/// Thin handler — delegates credential verification and session creation
/// to [`AuthService`](wardnetd_services::AuthService), then sets the session cookie.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful; session cookie is set", body = LoginResponse),
        (status = 400, description = "Malformed request body", body = ApiError),
        (status = 401, description = "Invalid credentials", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    security(()),
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let result = state
        .auth_service()
        .login(&body.username, &body.password)
        .await?;

    let cookie_value = format!(
        "wardnet_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        result.token, result.max_age_seconds
    );

    Ok((
        [(SET_COOKIE, cookie_value)],
        Json(LoginResponse {
            message: "logged in".to_owned(),
        }),
    ))
}
