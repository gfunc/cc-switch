# Design: AUTH_TOKEN Redesign

## Summary

Rename `JWT_SECRET` to `AUTH_TOKEN` and treat it as the admin login credential rather than a signing key for pre-issued tokens. The server generates one at startup if not set, persists it to a file, and the `cc-switch-web generate-token` CLI rotates it. The web UI login page accepts the auth token directly (replacing the JWT paste field) and removes the Reveal Token button.

## Motivation

The current flow is confusing: `JWT_SECRET` is a signing key, the user is expected to have a JWT, and the Reveal Token button generates one using the CLI flow. The natural model is: the admin has a single secret (the auth token), enters it on the login page, and the server issues a session JWT.

## Goals

1. Single credential: `AUTH_TOKEN` is the admin password and the JWT signing key.
2. Zero-config: server generates and persists `AUTH_TOKEN` at startup if not set.
3. Rotation: `cc-switch-web generate-token` rotates `AUTH_TOKEN`, invalidating existing sessions.
4. Simplified UI: remove the Reveal Token button; the user pastes their auth token directly.

## Non-goals

- Multi-user support (still single admin).
- Token expiration policies beyond the existing 24-hour JWT lifetime.
- Migration shim for existing `JWT_SECRET` users (hard rename).

## Architecture

The change touches the Rust web auth layer and the frontend login page. The JWT revocation blocklist and logout flow from the previous spec stay unchanged.

## Data Flow

### Startup

1. Server reads `AUTH_TOKEN` env var.
2. If unset, generate a 32-byte random hex string, write to `~/.config/cc-switch/auth_token` with `0600` permissions, and log the value to stderr once.
3. If set, use it directly (skip file write).

### Login

1. User opens web UI, sees password field labeled "Auth Token".
2. User pastes their `AUTH_TOKEN` value and submits.
3. Client sends `POST /auth/login` with `{ "token": "<submitted>" }`.
4. Server compares submitted value to `AUTH_TOKEN` using constant-time comparison.
5. On match: server generates a JWT via `generate_token("admin")` and returns it.
6. On mismatch: server returns 401.
7. Browser stores JWT in `localStorage` and uses it as a Bearer token for subsequent API calls.

### Rotation via CLI

1. Admin runs `cc-switch-web generate-token`.
2. CLI generates a new random `AUTH_TOKEN`, writes to the file, logs the new value to stderr.
3. CLI generates a JWT signed with the new `AUTH_TOKEN` and prints it.
4. All existing JWTs (signed with the old `AUTH_TOKEN`) immediately fail validation. Users must re-login with the new token.

## Backend Design

### `AUTH_TOKEN` loading

Replace the current `JWT_SECRET` static in `src-tauri/src/web/middleware/auth.rs`:

- Rename `JWT_SECRET` → `AUTH_TOKEN` (env var).
- On first access, read `AUTH_TOKEN` from env or from the persisted file.
- If neither exists, generate a random token, write to file, and log to stderr.
- Expose helpers: `get_auth_token() -> String` and `rotate_auth_token() -> String`.

### JWT signing

JWTs are signed using `AUTH_TOKEN` bytes directly as the HS256 key. Since `AUTH_TOKEN` is a high-entropy random string (or user-provided secret), this is safe. No HKDF needed.

### `POST /auth/login` endpoint

In `src-tauri/src/web/routes/auth.rs`:

- Request: `LoginRequest { token: String }`.
- Handler validates the submitted token against `AUTH_TOKEN` using `subtle::ConstantTimeEq` (or manual constant-time comparison).
- On match: calls `generate_token("admin")` and returns the JWT.
- On mismatch: returns `ApiResponse::error("Invalid auth token")` with HTTP 401.

### Removed endpoints

- `POST /auth/verify` — deleted.
- `POST /auth/generate` — deleted.
- `GET /auth/token-reveal-enabled` — deleted.
- `token_reveal_enabled()` helper — deleted.

### Removed env var

`CC_SWITCH_ENABLE_TOKEN_REVEAL` is no longer read.

### File path

`AUTH_TOKEN` file location: `~/.config/cc-switch/auth_token` (Linux/macOS), `%APPDATA%\cc-switch\auth_token` (Windows). Use the `dirs` crate which is already a dependency.

## Frontend Design

### LoginPage changes

In `src/components/auth/LoginPage.tsx`:

- Update field label: `t("login.token")` defaultValue "Auth Token" (was "Admin Token").
- Update placeholder: `t("login.tokenPlaceholder")` defaultValue "Paste your auth token here" (was "Paste your admin token here").
- Remove `useEffect` that fetches `isTokenRevealEnabled()`.
- Remove `tokenRevealEnabled` state and the conditional reveal button.
- On submit, call `POST /auth/login` with `{ token: token.trim() }`, expect `{ token: "<jwt>" }`, call `setAuthToken(jwt)`.

### API changes

- Remove `isTokenRevealEnabled` from `webAuthApi` and `authApi`.
- Keep `logout()` unchanged.
- Keep `generateWebAdminToken()` removed (no longer needed).

## Error Handling

- **Wrong auth token on login:** 401 with generic message. No distinction between "wrong token" and "server error" to avoid info leakage.
- **File write failure at startup:** log error and exit non-zero. The server cannot function without `AUTH_TOKEN`.
- **File write failure during `generate-token` rotation:** CLI exits non-zero; the in-memory token is not rotated.
- **Old JWT after rotation:** middleware signature validation fails → 401 → frontend `auth:expired` handler redirects to login.

## Testing Plan

### Backend tests

1. `AUTH_TOKEN` loaded from env var when set.
2. `AUTH_TOKEN` generated and persisted when env var unset (test with tempdir).
3. `POST /auth/login` returns JWT on valid token.
4. `POST /auth/login` returns 401 on invalid token.
5. `generate_token` produces JWTs with valid `jti` claim (existing test).
6. After `rotate_auth_token`, old JWTs fail validation.
7. Constant-time comparison prevents timing attacks (smoke test).

### Frontend tests

1. LoginPage submits to `/auth/login` (not `/auth/verify`).
2. LoginPage stores the returned JWT via `setAuthToken`.
3. LoginPage shows "Auth Token" label.
4. LoginPage no longer fetches `isTokenRevealEnabled`.
5. Existing logout tests pass unchanged.

## Open Questions

None — all clarifications resolved during brainstorming.
