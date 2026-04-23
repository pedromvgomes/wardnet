# Auth model

## Setup flow

- On first run, no admin exists. `GET /api/setup/status` returns `setup_completed: false`. Web UI redirects to setup page.
- `POST /api/setup` creates the first admin (Argon2id hashed). Returns 409 if already completed.

## Unauthenticated endpoints

- `GET /api/info` — version + uptime
- `GET /api/setup/status`, `POST /api/setup`
- `GET /api/devices/me`, `PUT /api/devices/me/rule` — self-service, identifies the caller by source IP via `ConnectInfo<SocketAddr>`

## Admin endpoints

Everything else. Requires an admin identity resolved from either a
session cookie (set by `POST /api/auth/login`) or an API key
(`Authorization: Bearer <key>`).

## Authentication context in services (HARD REQUIREMENT)

Every service method **must** validate the authentication context as its first operation using `auth_context::require_admin()?;` or `auth_context::require_authenticated()?;`. Services never trust their caller — they always check. This is a defense-in-depth measure: even if a handler bug exposes a service method, the auth guard inside the service rejects unauthorized calls.

### HTTP request path (automatic)

The `AuthContextLayer` middleware resolves the caller identity (from session cookie or API key) and sets a task-local `AuthContext` before the request reaches handlers. Service methods read it via `auth_context::require_admin()`.

### Background tasks calling services

Background processes (e.g. `IdleTunnelWatcher` tearing down idle tunnels, DHCP lease expiry, backup cleanup) run outside the HTTP middleware, so no `AuthContext` is set by default. They **must** wrap service calls in `auth_context::with_context()` to establish an admin identity:

```rust
use wardnet_common::auth::AuthContext;

// Background task calling a service method:
let admin_ctx = AuthContext::Admin { admin_id: Uuid::nil() };
auth_context::with_context(admin_ctx, tunnel_service.tear_down(id, "idle timeout")).await?;
```

Use `Uuid::nil()` (all zeros) as the `admin_id` for system-initiated operations — this clearly distinguishes background/system actions from real admin sessions in audit logs.

### Tests

Use the same `auth_context::with_context()` pattern to set the auth context before calling service methods:

```rust
let admin_ctx = AuthContext::Admin { admin_id: Uuid::new_v4() };
let result = auth_context::with_context(admin_ctx, svc.get_config()).await;
```

### Rules

1. Every service trait method implementation must call `auth_context::require_admin()?;` or `auth_context::require_authenticated()?;` as its first line.
2. The only exception is startup/restore methods that run before the system is ready (e.g. `restore_tunnels`) — these should be documented with a comment explaining why the guard is skipped.
3. Background tasks wrap service calls in `auth_context::with_context(AuthContext::Admin { admin_id: Uuid::nil() }, ...)`.
4. Tests wrap service calls in `auth_context::with_context(admin_ctx, ...)` to simulate the caller identity.
5. Anonymous callers get `Err(AppError::Forbidden)` — never silently succeed.
