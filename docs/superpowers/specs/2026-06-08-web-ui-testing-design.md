# Web UI Testing — Design Document

## Objective

Close the highest-priority test gaps identified in `.playwright-cli/review-web-vs-app.md`:

1. Unit tests for `LoginPage` + `web-client` (web entry point — any regression breaks the admin page)
2. One E2E smoke test via Playwright against a running `headless --enable-web` instance

## Scope

### In Scope

- Refactor `LoginPage.tsx` to use `web-client.post()` for auth verify (consistency with other web APIs)
- Unit tests for `src/lib/api/web-client.ts`
- Unit tests for `src/components/auth/LoginPage.tsx`
- Minimal Playwright E2E setup (`playwright.config.ts`, `tests/e2e/`)
- One smoke E2E test: login → dashboard → tab switch
- `package.json` `test:e2e` script

### Out of Scope

- `src/lib/api/index.ts` runtime branch (already covered by `tests/lib/apiRuntimeSelection.test.ts`)
- `src/lib/environment.ts` (lower priority per review doc)
- Other `src/lib/api/web/*.ts` files (12 files, lower priority)
- `WebServerSettings`, `useDirectorySettings`, `omo.ts`, `env.ts`, `updater.ts` (lower priority)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Unit Tests (vitest + jsdom + @testing-library/react)       │
├─────────────────────────────────────────────────────────────┤
│  tests/lib/web-client.test.ts                               │
│    ├─ fetchWithAuth: Bearer injection                       │
│    ├─ fetchWithAuth: 401 → clearAuthToken + redirect        │
│    ├─ fetchWithAuth: 401 on /login → no redirect loop       │
│    ├─ parseApiEnvelope: success / error / empty / bad JSON  │
│    └─ get/post/put/del: method routing + data extraction    │
│                                                             │
│  tests/components/auth/LoginPage.test.tsx                   │
│    ├─ Render: form, input, buttons visible                  │
│    ├─ Empty submit → error toast                            │
│    ├─ Reveal token → success + fills input                  │
│    ├─ Reveal token → error toast on failure                 │
│    ├─ Valid token submit → setAuthToken + onLogin           │
│    ├─ Invalid token → error toast, no onLogin               │
│    ├─ Network error → error toast                           │
│    └─ Loading states: buttons disabled during async         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  E2E Tests (Playwright)                                     │
├─────────────────────────────────────────────────────────────┤
│  playwright.config.ts                                       │
│    └─ Target: http://localhost:13001, headless Chromium     │
│                                                             │
│  tests/e2e/web-smoke.spec.ts                                │
│    1. Spawn: pnpm headless:debug:web (background process)   │
│    2. Wait: Server ready (poll /health)                     │
│    3. Generate: Token via CLI (cc-switch-web generate-token)│
│    4. Navigate: Go to base URL                              │
│    5. Login: Fill token, submit                             │
│    6. Verify: Dashboard renders (AppSwitcher tabs visible)  │
│    7. Switch: Click provider tab, verify content loads      │
│    8. Cleanup: Kill server process                          │
└─────────────────────────────────────────────────────────────┘
```

## LoginPage Refactor

### Current Behavior
`LoginPage` does raw `fetch` to `/api/v1/auth/verify` and manually parses the JSON envelope.

### New Behavior
Import `post` from `@/lib/api/web-client` and call:
```ts
const { valid } = await post("/auth/verify", { token: token.trim() });
```

This centralizes error handling and envelope parsing in `web-client.ts`.

## web-client.ts Test Plan

### Test Matrix

| Function | Scenario | Expected |
|----------|----------|----------|
| `setAuthToken` | Store token | `localStorage.setItem` called, `getAuthToken` returns it |
| `clearAuthToken` | Remove token | `localStorage.removeItem` called, `getAuthToken` returns null |
| `fetchWithAuth` | Token exists | `Authorization: Bearer <token>` header present |
| `fetchWithAuth` | Token is null | No `Authorization` header |
| `fetchWithAuth` | 401 on `/dashboard` | `clearAuthToken` called, redirect to `/login`, throws "Unauthorized" |
| `fetchWithAuth` | 401 on `/login` | `clearAuthToken` called, NO redirect, throws "Unauthorized" |
| `parseApiEnvelope` | Valid success envelope | Returns `{ success, data }` |
| `parseApiEnvelope` | `success: false` | Throws with `error` field message |
| `parseApiEnvelope` | Empty body | Throws with HTTP status label |
| `parseApiEnvelope` | Invalid JSON | Throws with HTTP status label |
| `get` | 200 success | Returns `data` field |
| `post` | 200 success | Sends POST, returns `data` field |
| `put` | 200 success | Sends PUT, returns `data` field |
| `del` | 200 success | Sends DELETE, returns `data` field |

## E2E Smoke Test Plan

### Server Lifecycle (auto-start)

Use Playwright's `globalSetup` + `globalTeardown` or a `test.beforeAll` / `test.afterAll` pattern:

1. **Start**: Spawn `AUTH_TOKEN=e2e-test-token CC_SWITCH_WEB_PORT=13002 CC_SWITCH_DB_PATH=/tmp/cc-switch-e2e.db pnpm headless:debug:web` as a background subprocess (see `playwright.config.ts` — the env vars are required, not optional)
2. **Wait**: Poll `http://localhost:13002/health` until `status === "healthy"` (timeout 30s)
3. **Token**: The static `AUTH_TOKEN=e2e-test-token` unlocks `POST /api/v1/auth/generate`; use it to mint a session JWT for the login flow
4. **Run tests**
5. **Cleanup**: SIGTERM the server process, wait for exit

### Test Steps

```
1. Navigate to http://localhost:13002
2. Assert: Login page visible ("CC Switch" heading, token input)
3. Fill token input with generated token (from `/auth/generate` using the static `AUTH_TOKEN`)
4. Click "Sign In"
5. Assert: Redirected to dashboard (URL no longer /login)
6. Assert: AppSwitcher tabs visible (Claude / Codex / Gemini)
7. Click "Claude" tab (if not default)
8. Assert: Provider list or settings content loads
```

### Error Handling

- Server start timeout → fail fast with clear message
- Token generation failure → skip test with diagnostic
- Login failure → screenshot on failure, include page HTML in error

## Data Flow

### Unit Test Flow

```
Test File → import { post } from "@/lib/api/web-client"
        → vi.spyOn(globalThis, "fetch") → mock Response
        → assert headers, URL, method, return value
        
Test File → render(<LoginPage onLogin={mock} />)
        → fireEvent.click / type
        → assert mock calls, toast messages, DOM state
```

### E2E Test Flow

```
Playwright Worker → globalSetup.spawn("AUTH_TOKEN=e2e-test-token CC_SWITCH_WEB_PORT=13002 ... pnpm headless:debug:web")
                → poll /health
                → POST /auth/generate with AUTH_TOKEN → session JWT
                → page.goto("http://localhost:13002")
                → page.fill("[id=token]", token)
                → page.click("button[type=submit]")
                → expect(page).toHaveURL(/^(?!.*login)/)
                → expect(page.getByText("Claude")).toBeVisible()
                → globalTeardown.kill(server)
```

## Testing Strategy

- **Unit tests**: Red-green-refactor. Write failing tests first, then implement/fix.
- **E2E test**: May need iterative debugging against real server. Use `--debug` flag for Playwright when developing.
- **CI consideration**: `headless:debug:web` requires a debug build. Document that E2E needs `cargo build` first. Not blocking for this PR.

## Edge Cases

| Edge | Handling |
|------|----------|
| Token with whitespace | `token.trim()` in LoginPage (already present, preserve) |
| Double-submit | Button `disabled={isLoading}` (already present, preserve) |
| 401 during E2E | Test should fail with screenshot; server may need restart between runs |
| Server port already in use | Use random port or check before spawn; document manual cleanup |
| Playwright browser download | First run needs `npx playwright install chromium` |

## Files Changed

### Modified
- `src/components/auth/LoginPage.tsx`
- `package.json`

### New
- `tests/lib/web-client.test.ts`
- `tests/components/auth/LoginPage.test.tsx`
- `playwright.config.ts`
- `tests/e2e/web-smoke.spec.ts`

## Success Criteria

- [ ] `pnpm test:unit` passes with new tests
- [ ] `LoginPage.tsx` uses `web-client.post()` for auth verify
- [ ] `tests/lib/web-client.test.ts` covers all 11 scenarios in matrix
- [ ] `tests/components/auth/LoginPage.test.tsx` covers all 8 scenarios
- [ ] `playwright.config.ts` exists and is valid
- [ ] `pnpm test:e2e` runs the smoke test (may require server build)
- [ ] All existing tests still pass
