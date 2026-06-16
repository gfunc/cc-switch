# Logout Button and Token Reveal Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a header logout button that server-side invalidates the current JWT and returns the user to the login page, and make the login page's "Reveal Token" button respect the `CC_SWITCH_ENABLE_TOKEN_REVEAL` server setting.

**Architecture:** Extend the Rust JWT auth middleware with a `jti` claim and an in-memory revocation blocklist; add `/auth/logout` and `/auth/token-reveal-enabled` routes. On the frontend, expose the new endpoints through `authApi`, render a web-only logout icon next to the Settings icon, and conditionally render the reveal button in `LoginPage` based on the server flag.

**Tech Stack:** Rust (axum, jsonwebtoken, uuid), TypeScript/React (Vite, Vitest, Testing Library, MSW), pnpm.

---

## File structure

| File | Responsibility |
|------|----------------|
| `src-tauri/src/web/middleware/auth.rs` | JWT claims, generation, validation, revocation blocklist, auth middleware. |
| `src-tauri/src/web/routes/auth.rs` | `/auth/logout` and `/auth/token-reveal-enabled` route handlers. |
| `src/lib/api/web/auth.ts` | Web-specific auth API helpers (`logout`, `isTokenRevealEnabled`). |
| `src/lib/api/auth.ts` | Aggregated `authApi` re-export so UI code imports from `@/lib/api`. |
| `src/App.tsx` | Header logout button and confirmation dialog. |
| `src/components/auth/LoginPage.tsx` | Hide "Reveal Token" button when server disables it. |
| `tests/components/auth/LoginPage.test.tsx` | Tests for reveal-button visibility. |
| `tests/integration/App.test.tsx` | Tests for logout button rendering and confirmation flow. |

---

## Task 1: Add `jti` claim and revocation blocklist to auth middleware

**Files:**
- Modify: `src-tauri/src/web/middleware/auth.rs`
- Test: `src-tauri/src/web/middleware/auth.rs` (inline `#[cfg(test)]` module)

- [ ] **Step 1: Write the failing tests**

Append a test module at the bottom of `src-tauri/src/web/middleware/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_contains_jti() {
        let token = generate_token("admin").expect("token generation failed");
        let claims = validate_token(&token).expect("token validation failed");
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn revoked_jti_is_rejected() {
        let token = generate_token("admin").expect("token generation failed");
        let jti = validate_token(&token).unwrap().jti;
        assert!(!is_jti_revoked(&jti));
        revoke_jti(jti.clone());
        assert!(is_jti_revoked(&jti));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::middleware::auth::tests
```

Expected: compilation errors because `jti`, `revoke_jti`, and `is_jti_revoked` do not exist.

- [ ] **Step 3: Implement the `jti` claim and blocklist helpers**

Modify `src-tauri/src/web/middleware/auth.rs`:

1. Add imports at the top:

```rust
use std::collections::HashSet;
use std::sync::Mutex;
```

2. Add blocklist static below `JWT_SECRET`:

```rust
/// In-memory set of revoked JWT IDs. Cleared on process restart.
static REVOKED_JTIS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn revoked_jtis() -> &'static Mutex<HashSet<String>> {
    REVOKED_JTIS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn revoke_jti(jti: String) {
    let mut set = revoked_jtis().lock().expect("revoked jti lock poisoned");
    set.insert(jti);
}

pub fn is_jti_revoked(jti: &str) -> bool {
    let set = revoked_jtis().lock().expect("revoked jti lock poisoned");
    set.contains(jti)
}

/// Revoke a token by extracting its `jti` claim. Returns the claims on success.
pub fn revoke_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let claims = validate_token(token)?;
    revoke_jti(claims.jti.clone());
    Ok(claims)
}
```

3. Add `jti` to `Claims`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}
```

4. Update `generate_token` to include a `jti`:

```rust
pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + TOKEN_EXPIRATION_SECONDS;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
```

5. Update `auth_middleware` to reject revoked tokens:

```rust
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Response<Body> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(json!({"error": "Missing or invalid authorization header"}).to_string()))
                .unwrap();
        }
    };

    let claims = match validate_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(json!({"error": "Invalid token"}).to_string()))
                .unwrap();
        }
    };

    if is_jti_revoked(&claims.jti) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(json!({"error": "Token has been revoked"}).to_string()))
            .unwrap();
    }

    next.run(request).await
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::middleware::auth::tests
```

Expected: both tests pass.

- [ ] **Step 5: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src-tauri/src/web/middleware/auth.rs
git commit -m "feat(auth): add jti claim and in-memory token revocation blocklist"
```

---

## Task 2: Add logout and token-reveal-status endpoints

**Files:**
- Modify: `src-tauri/src/web/routes/auth.rs`
- Test: `src-tauri/src/web/routes/auth.rs` (inline `#[cfg(test)]` module, optional but recommended)

- [ ] **Step 1: Write the failing tests**

Append a test module at the bottom of `src-tauri/src/web/routes/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::middleware::auth::{generate_token, validate_token, is_jti_revoked};
    use serial_test::serial;

    #[test]
    #[serial]
    fn token_reveal_enabled_reflects_env() {
        // Default (env unset) should be false.
        assert_eq!(token_reveal_enabled(), false);
    }

    #[tokio::test]
    async fn logout_route_revokes_token() {
        let token = generate_token("admin").unwrap();
        let jti = validate_token(&token).unwrap().jti;

        // Build a request with the Bearer token.
        let request = axum::http::Request::builder()
            .uri("/auth/logout")
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .body(axum::body::Body::empty())
            .unwrap();

        let response = logout_route(axum::extract::Request::from(request)).await;
        assert!(response.0.success);
        assert!(is_jti_revoked(&jti));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::routes::auth::tests
```

Expected: compilation errors because `logout_route` and the updated `ApiResponse` extraction do not exist.

- [ ] **Step 3: Implement the route handlers**

Modify `src-tauri/src/web/routes/auth.rs`:

1. Import the revocation helper:

```rust
use crate::web::middleware::auth::{validate_token, revoke_token};
```

2. Update `routes()`:

```rust
pub fn routes() -> Router {
    Router::new()
        .route("/verify", post(verify_token))
        .route("/generate", post(generate_token_route))
        .route("/login", post(login_deprecated))
        .route("/logout", post(logout_route))
        .route("/token-reveal-enabled", get(token_reveal_enabled_route))
}
```

3. Add the response structs and handlers below the existing handlers:

```rust
async fn logout_route(
    request: axum::extract::Request,
) -> Json<ApiResponse<()>> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Json(ApiResponse::error(
                "Missing or invalid authorization header".to_string(),
            ));
        }
    };

    match revoke_token(token) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Invalid token".to_string())),
    }
}

async fn token_reveal_enabled_route() -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(token_reveal_enabled()))
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::routes::auth::tests
```

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src-tauri/src/web/routes/auth.rs
git commit -m "feat(auth): add logout and token-reveal-enabled endpoints"
```

---

## Task 3: Extend frontend auth API with logout and token-reveal status

**Files:**
- Modify: `src/lib/api/web/auth.ts`
- Modify: `src/lib/api/auth.ts`
- Test: `tests/components/auth/LoginPage.test.tsx` (will use the new API in Task 5)

- [ ] **Step 1: Add methods to web auth API**

Modify `src/lib/api/web/auth.ts`:

```ts
import { post } from "../web-client";
import { get } from "../web-client";

export const webAuthApi = {
  async generateToken(): Promise<string> {
    return post("/auth/generate", {});
  },
  async logout(): Promise<void> {
    return post("/auth/logout", {});
  },
  async isTokenRevealEnabled(): Promise<boolean> {
    return get("/auth/token-reveal-enabled");
  },
};
```

Note: `post("/auth/logout", {})` automatically includes the current `Authorization` header from `web-client.ts`, so no token argument is needed.

- [ ] **Step 2: Expose methods through aggregated `authApi`**

Modify `src/lib/api/auth.ts`:

```ts
import { isTauri } from "@/lib/environment";
import { webAuthApi } from "./web/auth";

// ... existing managed auth functions ...

export async function generateWebAdminToken(): Promise<string> {
  if (isTauri()) {
    return invoke<string>("generate_web_token");
  }
  return webAuthApi.generateToken();
}

export async function logout(): Promise<void> {
  if (isTauri()) {
    // Tauri has no web login flow; no-op.
    return;
  }
  return webAuthApi.logout();
}

export async function isTokenRevealEnabled(): Promise<boolean> {
  if (isTauri()) {
    // Desktop login page is never shown; default to false.
    return false;
  }
  return webAuthApi.isTokenRevealEnabled();
}

export const authApi = {
  authStartLogin,
  authPollForAccount,
  authListAccounts,
  authGetStatus,
  authRemoveAccount,
  authSetDefaultAccount,
  authLogout,
  generateWebAdminToken,
  logout,
  isTokenRevealEnabled,
};
```

- [ ] **Step 3: Run typecheck**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm typecheck
```

Expected: no TypeScript errors.

- [ ] **Step 4: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src/lib/api/web/auth.ts src/lib/api/auth.ts
git commit -m "feat(auth): expose logout and token-reveal-enabled in authApi"
```

---

## Task 4: Add logout button and confirmation dialog to App header

**Files:**
- Modify: `src/App.tsx`
- Test: `tests/integration/App.test.tsx`

- [ ] **Step 1: Write the failing test**

Modify `tests/integration/App.test.tsx`:

1. Add a new mock helper for `ConfirmDialog` that exposes the title:

Replace the existing `ConfirmDialog` mock with:

```tsx
vi.mock("@/components/ConfirmDialog", () => ({
  ConfirmDialog: ({ isOpen, title, onConfirm, onCancel }: any) =>
    isOpen ? (
      <div data-testid="confirm-dialog">
        <span data-testid="confirm-dialog-title">{title}</span>
        <button onClick={() => onConfirm()}>confirm</button>
        <button onClick={() => onCancel()}>cancel</button>
      </div>
    ) : null,
}));
```

2. Add logout-specific tests at the end of the `describe` block:

```tsx
  it("renders logout button in web mode", async () => {
    const { default: App } = await import("@/App");
    renderApp(App);

    await waitFor(() =>
      expect(screen.getByTestId("provider-list").textContent).toContain(
        "claude-1",
      ),
    );

    expect(screen.getByTitle("common.logout")).toBeInTheDocument();
  });

  it("opens logout confirmation when logout button is clicked", async () => {
    const { default: App } = await import("@/App");
    renderApp(App);

    await waitFor(() =>
      expect(screen.getByTestId("provider-list").textContent).toContain(
        "claude-1",
      ),
    );

    fireEvent.click(screen.getByTitle("common.logout"));

    await waitFor(() =>
      expect(screen.getByTestId("confirm-dialog-title").textContent).toBe(
        "logout.confirmTitle",
      ),
    );
  });
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit tests/integration/App.test.tsx
```

Expected: tests fail because the logout button and title keys do not exist.

- [ ] **Step 3: Implement the logout UI in App.tsx**

Modify `src/App.tsx`:

1. Import `LogOut` from `lucide-react`:

```ts
import {
  Plus,
  Settings,
  ArrowLeft,
  Minus,
  Maximize2,
  Minimize2,
  X,
  Book,
  Brain,
  Wrench,
  RefreshCw,
  History,
  BarChart2,
  Download,
  FolderArchive,
  Search,
  FolderOpen,
  KeyRound,
  Shield,
  Cpu,
  LayoutDashboard,
  LogOut,
} from "lucide-react";
```

2. Import `authApi`, `isWebMode`, and `clearAuthToken`:

```ts
import { authApi } from "@/lib/api";
import { isWebMode } from "@/lib/environment";
import { clearAuthToken } from "@/lib/api/web-client";
```

3. Add state for the logout confirmation dialog:

```ts
const [logoutConfirmOpen, setLogoutConfirmOpen] = useState(false);
```

4. Add the logout handler:

```ts
const handleLogout = async () => {
  setLogoutConfirmOpen(false);
  try {
    await authApi.logout();
  } catch (error) {
    toast.warning(
      t("logout.serverInvalidationWarning", {
        defaultValue:
          "Logged out locally, but server session invalidation may have failed.",
      }),
    );
  } finally {
    clearAuthToken();
    queryClient.clear();
    setIsAuthenticated(false);
  }
};
```

5. Add the logout button to the header, immediately after the Settings button (around line 1249):

```tsx
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => {
                    setSettingsDefaultTab("general");
                    setCurrentView("settings");
                  }}
                  title={t("common.settings")}
                  className="hover:bg-black/5 dark:hover:bg-white/5"
                >
                  <Settings className="w-4 h-4" />
                </Button>
                {isWebMode() && (
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => setLogoutConfirmOpen(true)}
                    title={t("common.logout")}
                    className="hover:bg-black/5 dark:hover:bg-white/5"
                  >
                    <LogOut className="w-4 h-4" />
                  </Button>
                )}
                <UpdateBadge
```

6. Add the logout confirmation dialog near the other dialogs at the bottom of the component:

```tsx
      <ConfirmDialog
        isOpen={logoutConfirmOpen}
        title={t("logout.confirmTitle", { defaultValue: "Log out?" })}
        message={t("logout.confirmMessage", {
          defaultValue:
            "This will invalidate your current session and return you to the login page.",
        })}
        confirmText={t("logout.confirmAction", { defaultValue: "Log out" })}
        variant="info"
        onConfirm={() => void handleLogout()}
        onCancel={() => setLogoutConfirmOpen(false)}
      />
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit tests/integration/App.test.tsx
```

Expected: the new logout tests pass; existing tests still pass.

- [ ] **Step 5: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src/App.tsx tests/integration/App.test.tsx
git commit -m "feat(ui): add web-only logout button and confirmation dialog"
```

---

## Task 5: Conditionally render "Reveal Token" button based on server setting

**Files:**
- Modify: `src/components/auth/LoginPage.tsx`
- Test: `tests/components/auth/LoginPage.test.tsx`

- [ ] **Step 1: Write the failing tests**

Modify `tests/components/auth/LoginPage.test.tsx`:

1. Add a mock for the new API method:

```ts
const isTokenRevealEnabledMock = vi.fn();

vi.mock("@/lib/api", () => ({
  authApi: {
    generateWebAdminToken: () => generateWebAdminTokenMock(),
    isTokenRevealEnabled: () => isTokenRevealEnabledMock(),
  },
}));
```

2. Reset the mock in `beforeEach`:

```ts
  beforeEach(() => {
    toastSuccessMock.mockReset();
    toastErrorMock.mockReset();
    postMock.mockReset();
    setAuthTokenMock.mockReset();
    generateWebAdminTokenMock.mockReset();
    isTokenRevealEnabledMock.mockReset();
  });
```

3. Add tests:

```tsx
  it("hides reveal token button when token reveal is disabled", async () => {
    isTokenRevealEnabledMock.mockResolvedValue(false);

    renderLoginPage();

    await waitFor(() => {
      expect(
        screen.queryByRole("button", { name: "login.revealToken" }),
      ).not.toBeInTheDocument();
    });
  });

  it("shows reveal token button when token reveal is enabled", async () => {
    isTokenRevealEnabledMock.mockResolvedValue(true);

    renderLoginPage();

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "login.revealToken" }),
      ).toBeInTheDocument();
    });
  });

  it("hides reveal token button when token reveal status fetch fails", async () => {
    isTokenRevealEnabledMock.mockRejectedValue(new Error("network error"));

    renderLoginPage();

    await waitFor(() => {
      expect(
        screen.queryByRole("button", { name: "login.revealToken" }),
      ).not.toBeInTheDocument();
    });
  });
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit tests/components/auth/LoginPage.test.tsx
```

Expected: new tests fail because `LoginPage` does not yet use `isTokenRevealEnabled`.

- [ ] **Step 3: Implement conditional reveal button in LoginPage**

Modify `src/components/auth/LoginPage.tsx`:

1. Import `useEffect`:

```ts
import { useState, useEffect } from "react";
```

2. Add state and fetch the setting:

```ts
export function LoginPage({ onLogin }: LoginPageProps) {
  const { t } = useTranslation();
  const [token, setToken] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isRevealingToken, setIsRevealingToken] = useState(false);
  const [tokenRevealEnabled, setTokenRevealEnabled] = useState<boolean | null>(
    null,
  );

  useEffect(() => {
    let cancelled = false;
    authApi
      .isTokenRevealEnabled()
      .then((enabled) => {
        if (!cancelled) setTokenRevealEnabled(enabled);
      })
      .catch(() => {
        if (!cancelled) setTokenRevealEnabled(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // ... rest of component
```

3. Wrap the reveal button in a conditional:

```tsx
                {tokenRevealEnabled === true && (
                  <Button
                    type="button"
                    variant="outline"
                    className="w-full"
                    onClick={handleRevealToken}
                    disabled={isLoading || isRevealingToken}
                  >
                    {isRevealingToken ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        {t("login.revealing", { defaultValue: "Generating..." })}
                      </>
                    ) : (
                      <>
                        <Eye className="mr-2 h-4 w-4" />
                        {t("login.revealToken", { defaultValue: "Reveal Token" })}
                      </>
                    )}
                  </Button>
                )}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit tests/components/auth/LoginPage.test.tsx
```

Expected: all LoginPage tests pass.

- [ ] **Step 5: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src/components/auth/LoginPage.tsx tests/components/auth/LoginPage.test.tsx
git commit -m "feat(auth): hide reveal-token button when disabled server-side"
```

---

## Task 6: Verify everything together

**Files:** None (verification only).

- [ ] **Step 1: Run all unit tests**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit
```

Expected: all frontend tests pass.

- [ ] **Step 2: Run Rust tests**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch
```

Expected: all Rust tests pass.

- [ ] **Step 3: Run typecheck and format check**

Run:

```bash
cd /home/jason/projects/cc-switch && pnpm typecheck && pnpm format:check
```

Expected: no type errors or formatting issues.

- [ ] **Step 4: Commit any fixes**

If any formatting fixes were needed:

```bash
cd /home/jason/projects/cc-switch
pnpm format
git add -A
git commit -m "style: apply formatting"
```

---

## Self-review checklist

- [ ] **Spec coverage:** Every spec requirement maps to a task:
  - Logout button next to Settings → Task 4.
  - Server-side token invalidation → Tasks 1 and 2.
  - Return to login page → Task 4 (`setIsAuthenticated(false)`).
  - `CC_SWITCH_ENABLE_TOKEN_REVEAL` hides reveal button → Tasks 2, 3, and 5.
- [ ] **Placeholder scan:** No `TBD`, `TODO`, or vague instructions remain in tasks.
- [ ] **Type consistency:** `authApi.logout()` has no arguments and returns `Promise<void>` everywhere; `authApi.isTokenRevealEnabled()` returns `Promise<boolean>` everywhere.
- [ ] **Test commands:** Each step includes the exact command to run and the expected outcome.

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-16-logout-button-token-reveal.md`.

Two execution options:

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - Execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.

Which approach would you like?
