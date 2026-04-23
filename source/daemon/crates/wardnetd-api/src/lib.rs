pub mod api;
pub mod openapi;
pub mod state;
pub mod web;

/// Returns the fully-merged `OpenAPI` document for the Wardnet daemon.
///
/// Runs the same handler-registration chain that [`api::router`] uses and
/// extracts only the spec half of `OpenApiRouter::split_for_parts`, so every
/// `#[utoipa::path]` annotation is included alongside the shared metadata
/// ([`openapi::ApiDoc`] — title, tags, security schemes).
///
/// Exposed so external tools (e.g. the `dump_openapi` binary in Phase 2) can
/// produce a static spec file without booting the full HTTP stack.
#[must_use]
pub fn api_doc() -> utoipa::openapi::OpenApi {
    let (_router, spec) = api::build_openapi_router().split_for_parts();
    spec
}

#[cfg(test)]
mod tests;
