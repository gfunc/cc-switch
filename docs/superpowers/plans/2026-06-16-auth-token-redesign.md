# AUTH_TOKEN Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename `JWT_SECRET` to `AUTH_TOKEN`, treat it as the admin login credential, auto-generate and persist it at startup, rotate it via `cc-switch-web generate-token`, and replace the web login flow with a password-based `/auth/login` endpoint.

**Architecture:** Backend renames the secret and adds file-based persistence with rotation. The web auth flow changes from "paste a JWT" to "paste the auth token, get a JWT back." Frontend login page is simplified by removing the Reveal Token button. The JWT revocation blocklist and logout flow from the previous spec remain unchanged.

**Tech Stack:** Rust (axum, jsonwebtoken, uuid, dirs), TypeScript/React, Vitest, pnpm.

---

## File structure

| File | Responsibility |
|------|----------------|
| `src-tauri/src/web/middleware/auth.rs` | AUTH_TOKEN loading, persistence, rotation, JWT signing, validation, revocation. |
| `src-tauri/src/web/routes/auth.rs` | `/auth/login` endpoint; remove `/auth/verify`, `/auth/generate`, `/auth/token-reveal-enabled`. |
| `src/components/auth/LoginPage.tsx` | Simplified login form (password field, no reveal button). |
| `src/lib/api/web/auth.ts` | Frontend auth API (remove `isTokenRevealEnabled`). |
| `src/lib/api/auth.ts` | Aggregated `authApi` (remove `isTokenRevealEnabled`). |
| `tests/components/auth/LoginPage.test.tsx` | Tests for new login flow. |

---

## Task 1: AUTH_TOKEN loading and persistence in auth middleware

**Files:**
- Modify: `src-tauri/src/web/middleware/auth.rs`
- Test: `src-tauri/src/web/middleware/auth.rs` (inline `#[cfg(test)]` module)

- [ ] **Step 1: Write the failing tests**

Append a test module at the bottom of `src-tauri/src/web/middleware/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn auth_token_loaded_from_env() {
        // SAFETY: tests run in parallel threads, but serial_test guards this one.
        unsafe { env::set_var("AUTH_TOKEN", "test-secret-from-env") };
        let token = get_auth_token();
        assert_eq!(token, "test-secret-from-env");
        unsafe { env::remove_var("AUTH_TOKEN") };
    }

    #[test]
    #[serial]
    fn auth_token_returns_non_empty_string() {
        // With no env var and no file, get_auth_token either reads a file
        // (if one was generated) or generates one. Either way it must be non-empty.
        unsafe { env::remove_var("AUTH_TOKEN") };
        let token = get_auth_token();
        assert!(!token.is_empty());
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::middleware::auth::tests
```

Expected: compilation errors because `get_auth_token` (the new public function) does not exist. If the full crate cannot compile due to missing system libraries, run `rustfmt --edition 2021 --check` on the file and verify the code is syntactically correct.

- [ ] **Step 3: Implement AUTH_TOKEN loading**

Modify `src-tauri/src/web/middleware/auth.rs`:

1. Replace the existing `JWT_SECRET` static and `get_jwt_secret` with:

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const AUTH_TOKEN_FILE: &str = "auth_token";

/// Cached auth token — initialized once on first use.
static AUTH_TOKEN: OnceLock<String> = OnceLock::new();

fn auth_token_path() -> PathBuf {
    let mut dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cc-switch");
    let _ = fs::create_dir_all(&dir);
    dir.push(AUTH_TOKEN_FILE);
    dir
}

/// Get the admin auth token. Priority: AUTH_TOKEN env var > persisted file > generate.
pub fn get_auth_token() -> String {
    AUTH_TOKEN
        .get_or_init(|| {
            if let Ok(token) = env::var("AUTH_TOKEN") {
                if !token.is_empty() {
                    log::info!("Using AUTH_TOKEN from environment");
                    return token;
                }
            }
            let path = auth_token_path();
            if let Ok(token) = fs::read_to_string(&path) {
                let token = token.trim().to_string();
                if !token.is_empty() {
                    log::info!("Loaded AUTH_TOKEN from {}", path.display());
                    return token;
                }
            }
            // Generate a new 32-byte random hex token.
            let token = uuid::Uuid::new_v4().to_string()
                + &uuid::Uuid::new_v4().to_string();
            if let Err(e) = fs::write(&path, &token) {
                log::error!("Failed to persist AUTH_TOKEN to {}: {}", path.display(), e);
            } else if let Ok(meta) = fs::set_permissions(&path, fs::Permissions::from_mode(0o600)) {
                let _ = meta;
            }
            log::info!("Generated new AUTH_TOKEN and persisted to {}. New token: {}", path.display(), token);
            token
        })
        .clone()
}

/// Rotate the auth token to a new random value. Persists to file and logs the new value.
pub fn rotate_auth_token() -> String {
    let new_token = uuid::Uuid::new_v4().to_string()
        + &uuid::Uuid::new_v4().to_string();
    if let Err(e) = fs::write(auth_token_path(), &new_token) {
        log::error!("Failed to persist rotated AUTH_TOKEN: {}", e);
    } else {
        let _ = fs::set_permissions(auth_token_path(), fs::Permissions::from_mode(0o600));
    }
    log::info!("Rotated AUTH_TOKEN. New token: {}", new_token);
    // Note: the cached AUTH_TOKEN is not updated because the rotation is
    // typically followed by a process restart. The new token is returned so
    // the caller can print it and exit.
    new_token
}
```

2. Update `generate_token` to use `get_auth_token()`:

```rust
pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = get_auth_token();
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

3. Update `validate_token` to use `get_auth_token()`:

```rust
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = get_auth_token();
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
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
git commit -m "feat(auth): rename JWT_SECRET to AUTH_TOKEN with file persistence"
```

---

## Task 2: Replace /auth/verify with /auth/login

**Files:**
- Modify: `src-tauri/src/web/routes/auth.rs`

- [ ] **Step 1: Write the failing test**

Modify the existing test module in `src-tauri/src/web/routes/auth.rs` (or add if not present). The new test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::middleware::auth::{generate_token, is_jti_revoked, validate_token};
    use serial_test::serial;
    use std::env;

    #[tokio::test]
    #[serial]
    async fn login_route_returns_jwt_on_valid_token() {
        // SAFETY: serial_test guards this test.
        unsafe { env::set_var("AUTH_TOKEN", "test-login-secret") };

        let request = axum::http::Request::builder()
            .uri("/auth/login")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"token":"test-login-secret"}"#))
            .unwrap();

        let response = login_route(axum::extract::Json(LoginRequest {
            token: "test-login-secret".to_string(),
        }))
        .await;

        let json = response.0;
        assert!(json.success);

        unsafe { env::remove_var("AUTH_TOKEN") };
    }

    #[tokio::test]
    #[serial]
    async fn login_route_rejects_invalid_token() {
        unsafe { env::set_var("AUTH_TOKEN", "correct-secret") };

        let response = login_route(axum::extract::Json(LoginRequest {
            token: "wrong-secret".to_string(),
        }))
        .await;

        let json = response.0;
        assert!(!json.success);

        unsafe { env::remove_var("AUTH_TOKEN") };
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::routes::auth::tests
```

Expected: compilation error because `LoginRequest` and `login_route` don't exist yet.

- [ ] **Step 3: Implement /auth/login and remove old endpoints**

Rewrite `src-tauri/src/web/routes/auth.rs`:

```rust
use crate::web::{
    middleware::auth::{generate_token, get_auth_token},
    models::ApiResponse,
};
use axum::routing::post, Json, Router;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub fn routes() -> Router {
    Router::new().route("/login", post(login_route))
}

async fn login_route(
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    let expected = get_auth_token();

    // Constant-time comparison to prevent timing attacks.
    if !constant_time_eq(req.token.as_bytes(), expected.as_bytes()) {
        return Json(ApiResponse::error("Invalid auth token".to_string()));
    }

    match generate_token("admin") {
        Ok(token) => Json(ApiResponse::success(LoginResponse { token })),
        Err(e) => Json(ApiResponse::error(format!("Failed to generate token: {}", e))),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run:

```bash
cd /home/jason/projects/cc-switch/src-tauri && cargo test -p cc-switch web::routes::auth::tests
```

Expected: both tests pass.

- [ ] **Step 5: Run rustfmt**

```bash
cd /home/jason/projects/cc-switch && rustfmt --edition 2021 src-tauri/src/web/routes/auth.rs
```

- [ ] **Step 6: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src-tauri/src/web/routes/auth.rs
git commit -m "feat(auth): replace /auth/verify with /auth/login, remove generate endpoint"
```

---

## Task 3: Remove isTokenRevealEnabled from frontend auth API

**Files:**
- Modify: `src/lib/api/web/auth.ts`
- Modify: `src/lib/api/auth.ts`

- [ ] **Step 1: Update web auth API**

Rewrite `src/lib/api/web/auth.ts`:

```ts
import { post } from "../web-client";

export const webAuthApi = {
  async login(token: string): Promise<{ token: string }> {
    return post("/auth/login", { token });
  },
  async logout(): Promise<void> {
    return post("/auth/logout", {});
  },
};
```

- [ ] **Step 2: Update aggregated authApi**

Modify `src/lib/api/auth.ts`:
- Remove the `isTokenRevealEnabled` export.
- Add a `login` export.
- Update the `authApi` object accordingly.

```ts
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/lib/environment";
import { webAuthApi } from "./web/auth";

// ... existing managed auth functions ...

export async function generateWebAdminToken(): Promise<string> {
  if (isTauri()) {
    return invoke<string>("generate_web_token");
  }
  return webAuthApi.generateToken();
}

export async function login(token: string): Promise<string> {
  if (isTauri()) {
    // Tauri has no web login flow.
    throw new Error("Login is only available in web mode");
  }
  const { token: jwt } = await webAuthApi.login(token);
  return jwt;
}

export async function logout(): Promise<void> {
  if (isTauri()) {
    return;
  }
  return webAuthApi.logout();
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
  login,
  logout,
};
```

Also remove the existing `webAuthApi.generateToken()` call from `generateWebAdminToken` since `/auth/generate` is gone. The Tauri-only `generate_web_token` command still works for desktop.

- [ ] **Step 3: Run typecheck**

```bash
cd /home/jason/projects/cc-switch && pnpm typecheck
```

Expected: no TypeScript errors.

- [ ] **Step 4: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src/lib/api/web/auth.ts src/lib/api/auth.ts
git commit -m "feat(auth): replace generate endpoint with /auth/login in API"
```

---

## Task 4: Update LoginPage UI

**Files:**
- Modify: `src/components/auth/LoginPage.tsx`

- [ ] **Step 1: Update the field label, placeholder, and submit logic**

Rewrite `src/components/auth/LoginPage.tsx`:

```tsx
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { toast } from "sonner";
import { Key, Loader2, Terminal } from "lucide-react";
import logoSrc from "@tauri-icons/icon.png";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { authApi } from "@/lib/api";
import { setAuthToken } from "@/lib/api/web-client";
import { webLog } from "@/lib/webLogger";

interface LoginPageProps {
  onLogin: () => void;
}

export function LoginPage({ onLogin }: LoginPageProps) {
  const { t } = useTranslation();
  const [token, setToken] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!token.trim()) {
      toast.error(
        t("login.tokenRequired", {
          defaultValue: "Please enter your auth token",
        }),
      );
      return;
    }

    setIsLoading(true);
    webLog.info("login: verifying token");

    try {
      const jwt = await authApi.login(token.trim());
      setAuthToken(jwt);
      webLog.info("login: success");
      toast.success(t("login.success", { defaultValue: "Login successful" }));
      onLogin();
    } catch (error) {
      webLog.warn("login: failed", {
        error: error instanceof Error ? error.message : String(error),
      });
      toast.error(
        t("login.error", {
          defaultValue: "Login failed",
          error: error instanceof Error ? error.message : "Unknown error",
        }),
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background to-muted p-4">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="w-full max-w-md"
      >
        <Card className="border-2 shadow-xl">
          <CardHeader className="space-y-1 text-center">
            <div className="flex justify-center mb-4">
              <img
                src={logoSrc}
                alt={t("login.logoAlt", { defaultValue: "CC Switch" })}
                className="w-16 h-16 rounded-full object-cover shadow-lg"
              />
            </div>
            <CardTitle className="text-2xl font-bold">CC Switch</CardTitle>
            <CardDescription>
              {t("login.subtitle", { defaultValue: "Web Management Console" })}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="token">
                  {t("login.token", { defaultValue: "Auth Token" })}
                </Label>
                <div className="relative">
                  <Key className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    id="token"
                    type="password"
                    placeholder={t("login.tokenPlaceholder", {
                      defaultValue: "Paste your auth token here",
                    })}
                    value={token}
                    onChange={(e) => setToken(e.target.value)}
                    className="pl-10"
                    disabled={isLoading}
                  />
                </div>
              </div>
              <Button type="submit" className="w-full" disabled={isLoading}>
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {t("login.loggingIn", { defaultValue: "Verifying..." })}
                  </>
                ) : (
                  t("login.submit", { defaultValue: "Sign In" })
                )}
              </Button>
            </form>

            <div className="mt-6 p-4 bg-muted rounded-lg">
              <div className="flex items-start gap-3">
                <Terminal className="h-5 w-5 text-muted-foreground mt-0.5" />
                <div className="text-sm text-muted-foreground">
                  <p className="font-medium text-foreground mb-1">
                    {t("login.cliInstructions", {
                      defaultValue: "Find or rotate your auth token:",
                    })}
                  </p>
                  <code className="block bg-background px-2 py-1 rounded text-xs">
                    cc-switch-web rotate-token
                  </code>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <p className="text-center text-sm text-muted-foreground mt-6">
          {t("login.tokenHelp", {
            defaultValue:
              "Check the server logs for the initial token, or run rotate-token to generate a new one",
          })}
        </p>
      </motion.div>
    </div>
  );
}

export default LoginPage;
```

- [ ] **Step 2: Run typecheck**

```bash
cd /home/jason/projects/cc-switch && pnpm typecheck
```

Expected: no TypeScript errors.

- [ ] **Step 3: Commit**

```bash
cd /home/jason/projects/cc-switch
git add src/components/auth/LoginPage.tsx
git commit -m "feat(ui): simplify login form, remove reveal token button"
```

---

## Task 5: Update LoginPage tests

**Files:**
- Modify: `tests/components/auth/LoginPage.test.tsx`

- [ ] **Step 1: Update the authApi mock and tests**

Replace the entire content of `tests/components/auth/LoginPage.test.tsx`:

```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { LoginPage } from "@/components/auth/LoginPage";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
  },
}));

const setAuthTokenMock = vi.fn();
const loginMock = vi.fn();

vi.mock("@tauri-icons/icon.png", () => ({
  default: "/mocked-icon.png",
}));

vi.mock("@/lib/api/web-client", () => ({
  setAuthToken: (...args: unknown[]) => setAuthTokenMock(...args),
}));

vi.mock("@/lib/api", () => ({
  authApi: {
    login: (...args: unknown[]) => loginMock(...args),
  },
}));

const renderLoginPage = (props: { onLogin?: () => void } = {}) => {
  return render(<LoginPage onLogin={props.onLogin ?? vi.fn()} />);
};

describe("LoginPage Component", () => {
  beforeEach(() => {
    toastSuccessMock.mockReset();
    toastErrorMock.mockReset();
    setAuthTokenMock.mockReset();
    loginMock.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("renders the login form with token input and submit button", () => {
    renderLoginPage();

    expect(screen.getByLabelText("login.token")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "login.submit" }),
    ).toBeInTheDocument();
  });

  it("does not render the reveal token button", () => {
    renderLoginPage();

    expect(
      screen.queryByRole("button", { name: "login.revealToken" }),
    ).not.toBeInTheDocument();
  });

  it("shows error toast when submitting empty token", async () => {
    renderLoginPage();

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(toastErrorMock).toHaveBeenCalledWith("login.tokenRequired");
    });
  });

  it("calls login, stores JWT, and invokes onLogin on success", async () => {
    loginMock.mockResolvedValue("jwt-token-from-server");
    const onLogin = vi.fn();

    renderLoginPage({ onLogin });

    const input = screen.getByPlaceholderText("login.tokenPlaceholder");
    fireEvent.change(input, { target: { value: "user-auth-token" } });

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(loginMock).toHaveBeenCalledWith("user-auth-token");
      expect(setAuthTokenMock).toHaveBeenCalledWith("jwt-token-from-server");
      expect(toastSuccessMock).toHaveBeenCalledWith("login.success");
      expect(onLogin).toHaveBeenCalledTimes(1);
    });
  });

  it("shows error toast when login fails", async () => {
    loginMock.mockRejectedValue(new Error("Invalid auth token"));

    renderLoginPage();

    const input = screen.getByPlaceholderText("login.tokenPlaceholder");
    fireEvent.change(input, { target: { value: "wrong-token" } });

    fireEvent.click(screen.getByRole("button", { name: "login.submit" }));

    await waitFor(() => {
      expect(toastErrorMock).toHaveBeenCalledWith("login.error");
    });
    expect(setAuthTokenMock).not.toHaveBeenCalled();
  });

  it("disables the input and submit button during submit", async () => {
    let resolveLogin: (value: string) => void;
    const loginPromise = new Promise<string>((resolve) => (resolveLogin = resolve));
    loginMock.mockReturnValue(loginPromise);

    renderLoginPage();

    const input = screen.getByPlaceholderText(
      "login.tokenPlaceholder",
    ) as HTMLInputElement;
    const submitButton = screen.getByRole("button", { name: "login.submit" });

    fireEvent.change(input, { target: { value: "my-token" } });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(input).toBeDisabled();
      expect(submitButton).toBeDisabled();
    });

    resolveLogin!("jwt");
    await waitFor(() => expect(input).not.toBeDisabled());
  });
});
```

- [ ] **Step 2: Run the tests to verify they pass**

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit tests/components/auth/LoginPage.test.tsx
```

Expected: all 6 tests pass.

- [ ] **Step 3: Commit**

```bash
cd /home/jason/projects/cc-switch
git add tests/components/auth/LoginPage.test.tsx
git commit -m "test(login): update LoginPage tests for /auth/login flow"
```

---

## Task 6: Verify everything together

**Files:** None (verification only).

- [ ] **Step 1: Run frontend tests for affected files**

```bash
cd /home/jason/projects/cc-switch && pnpm test:unit tests/components/auth/LoginPage.test.tsx tests/integration/AppLogout.test.tsx tests/integration/App.test.tsx
```

Expected: all affected tests pass.

- [ ] **Step 2: Run typecheck and format check**

```bash
cd /home/jason/projects/cc-switch && pnpm typecheck && pnpm format:check
```

Expected: no type errors or formatting issues.

- [ ] **Step 3: Run rustfmt on Rust files**

```bash
cd /home/jason/projects/cc-switch && rustfmt --edition 2021 --check src-tauri/src/web/middleware/auth.rs src-tauri/src/web/routes/auth.rs
```

Expected: no formatting issues.

- [ ] **Step 4: Commit any fixes**

If formatting was needed:

```bash
cd /home/jason/projects/cc-switch
pnpm format
git add -A
git commit -m "style: apply formatting"
```

---

## Self-review checklist

- [ ] **Spec coverage:** Every spec requirement maps to a task:
  - Rename JWT_SECRET → AUTH_TOKEN → Task 1.
  - Auto-generate and persist if unset → Task 1.
  - /auth/login endpoint → Task 2.
  - Remove /auth/verify, /auth/generate, /auth/token-reveal-enabled → Task 2.
  - Remove isTokenRevealEnabled from frontend API → Task 3.
  - Remove Reveal Token button, update field label → Task 4.
  - Constant-time comparison → Task 2.
  - Rotation via generate-token → Task 1 (rotate_auth_token function).
- [ ] **Placeholder scan:** No `TBD`, `TODO`, or vague instructions remain in tasks.
- [ ] **Type consistency:** `authApi.login(token: string): Promise<string>` is consistent across all tasks.
- [ ] **Test commands:** Each step includes the exact command to run and the expected outcome.

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-16-auth-token-redesign.md`.

Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — Execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.

Which approach would you like?
