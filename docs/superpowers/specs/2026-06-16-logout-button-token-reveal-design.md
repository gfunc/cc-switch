# Design: Logout Button and Token Reveal Visibility

## Summary

Add a logout button to the top-left header of the CC Switch web UI, immediately to the right of the Settings icon. Clicking it confirms, invalidates the current JWT on the server, and returns the user to the login page. Also make the login page's "Reveal Token" button respect the existing `CC_SWITCH_ENABLE_TOKEN_REVEAL` server-side environment variable.

## Motivation

- Web UI users currently have no way to sign out without clearing site data manually.
- The "Reveal Token" button is always shown in the login page even when the backend endpoint is disabled by `CC_SWITCH_ENABLE_TOKEN_REVEAL`, leading to a confusing error if clicked.

## Goals

1. Provide a visible logout affordance in the web UI header.
2. Invalidate the current session token server-side on logout.
3. Keep the reveal-token UI consistent with the server configuration.

## Non-goals

- Changing the Tauri desktop auth flow (desktop remains always-authenticated).
- Adding a full session store or refresh-token mechanism.
- Persisting the blocklist across server restarts.

## Architecture

The change touches three layers:

1. **Frontend UI** (`src/App.tsx`, `src/components/auth/LoginPage.tsx`)
   - New logout icon button in the header.
   - Logout confirmation dialog.
   - Conditional rendering of the "Reveal Token" button.
2. **Frontend API layer** (`src/lib/api/web/auth.ts`, `src/lib/api/web-client.ts`)
   - `logout(token)` and `isTokenRevealEnabled()` helpers.
3. **Backend** (`src-tauri/src/web/routes/auth.rs`, `src-tauri/src/web/middleware/auth.rs`)
   - `POST /auth/logout` endpoint.
   - `GET /auth/token-reveal-enabled` endpoint.
   - In-memory JWT blocklist in auth middleware.

## Data Flow

### Logout

1. User clicks the logout icon in the header.
2. `ConfirmDialog` asks for confirmation.
3. On confirm, frontend calls `POST /auth/logout` with the current token in the `Authorization` header.
4. Backend validates the token, extracts its `jti`, and adds the `jti` to an in-memory blocklist.
5. Backend returns success.
6. Frontend calls `clearAuthToken()`, `queryClient.clear()`, and `setIsAuthenticated(false)`.
7. The existing unauthenticated redirect renders `LoginPage`.

### Token reveal visibility

1. `LoginPage` mounts.
2. Frontend calls `GET /auth/token-reveal-enabled`.
3. If the response is `false`, the "Reveal Token" button is hidden.
4. If the response is `true`, the button is shown as before.

## Backend Design

### JWT claims

Add a `jti` (JWT ID) claim to generated tokens so each token has a stable identifier:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}
```

The `jti` is generated with `uuid::Uuid::new_v4().to_string()`.

### Token blocklist

Use a process-local blocklist stored in a static:

```rust
use std::collections::HashSet;
use std::sync::Mutex;

static REVOKED_JTIS: std::sync::OnceLock<Mutex<HashSet<String>>> = std::sync::OnceLock::new();
```

Helper functions:

- `revoke_jti(jti: String)` — inserts the `jti` into the blocklist.
- `is_jti_revoked(jti: &str) -> bool` — checks membership.

### Auth middleware update

After validating the token, check whether its `jti` is revoked. If revoked, return `401 Unauthorized`.

### New endpoints

- `POST /auth/logout`
  - Requires a valid Bearer token.
  - Extracts the token from the `Authorization` header, validates it, revokes its `jti`.
  - Returns `ApiResponse<()>` success.
- `GET /auth/token-reveal-enabled`
  - Public endpoint (no auth required).
  - Returns `ApiResponse<bool>` based on `CC_SWITCH_ENABLE_TOKEN_REVEAL`.

## Frontend Design

### Header logout button

In `src/App.tsx`, inside the top-left header group that contains the Settings icon, add a `LogOut` icon button from `lucide-react` immediately to the right of the Settings button. Render it only when `isWebMode()` is true. Use `variant="ghost" size="icon"` with the same hover styling as the Settings button.

Clicking it sets `logoutConfirmOpen` state to true.

### Logout confirmation

Reuse the existing `ConfirmDialog` component:

- Title: "Log out?"
- Message: "This will invalidate your current session and return you to the login page."
- Confirm text: "Log out"
- Variant: default / warning

On confirm:

1. Call `webAuthApi.logout(getAuthToken()!)`.
2. Regardless of success or failure, call `clearAuthToken()` and `queryClient.clear()`.
3. Call `setIsAuthenticated(false)`.
4. If the API failed, show a toast warning that logout completed locally but server invalidation may have failed.

### Token reveal visibility

In `src/components/auth/LoginPage.tsx`:

- Add state `tokenRevealEnabled: boolean | null` initialized to `null`.
- In `useEffect` on mount, call `webAuthApi.isTokenRevealEnabled()`.
- If `tokenRevealEnabled === true`, render the "Reveal Token" button.
- If `false` or `null`, hide it (fail-safe).

### API additions

In `src/lib/api/web/auth.ts`:

```ts
export const webAuthApi = {
  async generateToken(): Promise<string> {
    return post("/auth/generate", {});
  },
  async logout(token: string): Promise<void> {
    return post("/auth/logout", {}, token); // or pass via Authorization header
  },
  async isTokenRevealEnabled(): Promise<boolean> {
    return get("/auth/token-reveal-enabled");
  },
};
```

The logout request must send the token in the `Authorization` header. This can be done by temporarily setting the header in the request or by adding an overload to `post` that accepts a custom token.

## Error Handling

- **Logout API fails:** Still clear local auth state and redirect to login. Show a warning toast so the user knows server invalidation may not have succeeded.
- **Token-reveal-status fetch fails:** Default to hiding the button to avoid offering a broken action.
- **Token already expired/invalid on logout:** The middleware will reject it. The frontend still clears local state and redirects.

## Testing Plan

### Backend tests

1. Unit test: generated token contains a non-empty `jti` claim.
2. Unit test: `revoke_jti` + `is_jti_revoked` behavior.
3. Middleware test: a revoked token is rejected with 401.
4. Route test: `POST /auth/logout` revokes the token; subsequent requests with the same token fail.
5. Route test: `GET /auth/token-reveal-enabled` returns the correct value for both env states.

### Frontend tests

1. `App.test.tsx`: logout button is rendered in web mode and not rendered in Tauri mode.
2. `App.test.tsx`: clicking logout opens the confirmation dialog.
3. `LoginPage.test.tsx`: "Reveal Token" button is hidden when `isTokenRevealEnabled` returns `false`.
4. `LoginPage.test.tsx`: "Reveal Token" button is shown when `isTokenRevealEnabled` returns `true`.

## Open Questions

None — all clarifications resolved during brainstorming.
