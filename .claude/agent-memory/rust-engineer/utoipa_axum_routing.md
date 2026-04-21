---
name: utoipa-axum OpenApiRouter path semantics
description: utoipa-axum `routes!()` uses the handler's `#[utoipa::path(path = "...")]` verbatim; nesting double-prefixes
type: project
---

# utoipa-axum — `routes!()` path handling

`utoipa_axum::routes!(handler)` registers routes onto an `OpenApiRouter` using the `path` declared inside the handler's `#[utoipa::path(...)]` attribute **verbatim** (see `utoipa-axum-0.2.0/src/router.rs::routes`).

**Why:** This is nice for OpenAPI docs (the path shown in Scalar matches what the handler declares) but it means you cannot ALSO `.nest("/api", router)` — the outer nest prefixes the path again, producing `/api/api/...`.

**How to apply (in this repo):**
- Handler paths: declare the **full** path including `/api/` prefix, e.g. `#[utoipa::path(path = "/api/devices")]`. This is what users see in the generated docs.
- Router assembly in `api/mod.rs`: build an `OpenApiRouter`, call `.split_for_parts()` to get `(axum::Router, OpenApi)`, then `Router::new().merge(api_router).fallback(...)`. Do **not** `.nest("/api", ...)` — the `/api` is already baked into each route via `routes!()`.
- WebSocket / non-utoipa endpoints: attach with full `/api/...` path via `.route("/api/system/logs/stream", get(handler))` on the axum `Router` returned by `split_for_parts()`, keeping the prefix consistent.

## Per-module route registration convention

Each file under `crates/wardnetd-api/src/api/` exposes a `register(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState>` function. `api/mod.rs::router()` threads a single `OpenApiRouter` through `auth::register`, `dns::register`, etc. This keeps route declarations co-located with handlers instead of a giant route table.

## Typed Path/Query params

- `Path<Uuid>` params need `params(("id" = Uuid, Path, description = "..."))` on `#[utoipa::path]`.
- `Path<String>` (e.g. provider IDs) uses `params(("id" = String, Path, description = "..."))`.
- Query struct params need `#[derive(IntoParams)] #[into_params(parameter_in = Query)]` on the struct, then `params(HistoryQuery)` on the path.

## Local schema mirrors

If a response body type lives in a crate that does not depend on `utoipa` (e.g. `wardnetd_services::logging::error_notifier::ErrorEntry`), define a thin API-layer mirror struct with `#[derive(Serialize, ToSchema)]` in `api/<module>.rs` plus a `From<ServiceType>` impl. Avoids pulling `utoipa` into the services crate.
