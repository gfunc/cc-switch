# Web UI Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add unit tests for `LoginPage` and `web-client`, refactor `LoginPage` to use `web-client`, and add one Playwright E2E smoke test for web mode.

**Architecture:** `LoginPage` currently does raw `fetch` for auth — refactor to `web-client.post()` for consistency. Unit tests use vitest + jsdom + `@testing-library/react` with `vi.spyOn(fetch)` (matching existing `webProxyApi.test.ts` pattern). E2E uses Playwright with `webServer` auto-start against `pnpm headless:debug:web`.

**Tech Stack:** vitest, jsdom, @testing-library/react, MSW, Playwright, Rust/Tauri backend

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/components/auth/LoginPage.tsx` | Modify | Refactor auth verify to use `web-client.post()` |
| `tests/components/auth/LoginPage.test.tsx` | Create | Unit tests for LoginPage form, reveal token, submit, errors |
| `tests/lib/web-client.test.ts` | Create | Unit tests for `web-client.ts`: Bearer, 401, envelope parsing |
| `playwright.config.ts` | Create | Playwright config with `webServer` auto-start |
| `tests/e2e/web-smoke.spec.ts` | Create | E2E smoke: login → dashboard → switch tab |
| `package.json` | Modify | Add `test:e2e` script |

---

### Task 1: LoginPage Refactor + Unit Tests

**Files:**
- Modify: `src/components/auth/LoginPage.tsx`
- Create: `tests/components/auth/LoginPage.test.tsx`

**Context:** `LoginPage` currently imports `authApi` and `setAuthToken`. It uses raw `fetch` for the `/auth/verify` call. We refactor to use `post` from `@/lib/api/web-client`. The component renders a form with a token input, a "Reveal Token" button, and a "Sign In" submit button.

- [ ] **Step 1: Write failing LoginPage tests**

Create `tests/components/auth/LoginPage.test.tsx`:

```tsx
import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { LoginPage } from "@/components/auth/LoginPage";
import { authApi } from "@/lib/api";
import { post, setAuthToken } from "@/lib/api/web-client";

vi.mock("@/lib/api", () => ({
  authApi: {
    generateWebAdminToken: vi.fn(),
  },
}));

vi.mock("@/lib/api/web-client", async () => {
  const actual = await vi.importActual("@/lib/api/web-client");
  return {
    ...actual as object,
    post: vi.fn(),
    setAuthToken: vi.fn(),
  };
});

describe("LoginPage", () => {
  const mockOnLogin = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the login form with input and buttons", () => {
    render(<LoginPage onLogin={mockOnLogin} />);
    expect(screen.getByText("CC Switch")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Paste your admin token here/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Sign In/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Reveal Token/i })).toBeInTheDocument();
  });

  it("shows error toast when submitting empty token", async () => {
    render(<LoginPage onLogin={mockOnLogin} />);
    fireEvent.click(screen.getByRole("button", { name: /Sign In/i }));
    await waitFor(() => {
      expect(screen.getByText(/Please enter your admin token/i)).toBeInTheDocument();
    });
  });

  it("reveals token and fills input on success", async () => {
    vi.mocked(authApi.generateWebAdminToken).mockResolvedValue("revealed-token-123");
    render(<LoginPage onLogin={mockOnLogin} />);

    fireEvent.click(screen.getByRole("button", { name: /Reveal Token/i }));

    await waitFor(() => {
      expect(authApi.generateWebAdminToken).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.getByDisplayValue("revealed-token-123")).toBeInTheDocument();
    });
  });

  it("shows error toast when reveal token fails", async () => {
    vi.mocked(authApi.generateWebAdminToken).mockRejectedValue(new Error("Server error"));
    render(<LoginPage onLogin={mockOnLogin} />);

    fireEvent.click(screen.getByRole("button", { name: /Reveal Token/i }));

    await waitFor(() => {
      expect(screen.getByText(/Failed to reveal token/i)).toBeInTheDocument();
    });
  });

  it("submits valid token, calls setAuthToken and onLogin", async () => {
    vi.mocked(post).mockResolvedValue({ valid: true });
    render(<LoginPage onLogin={mockOnLogin} />);

    fireEvent.change(screen.getByPlaceholderText(/Paste your admin token here/i), {
      target: { value: "valid-token" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Sign In/i }));

    await waitFor(() => {
      expect(post).toHaveBeenCalledWith("/auth/verify", { token: "valid-token" });
    });
    await waitFor(() => {
      expect(setAuthToken).toHaveBeenCalledWith("valid-token");
    });
    await waitFor(() => {
      expect(mockOnLogin).toHaveBeenCalledTimes(1);
    });
  });

  it("shows error toast when token is invalid", async () => {
    vi.mocked(post).mockResolvedValue({ valid: false });
    render(<LoginPage onLogin={mockOnLogin} />);

    fireEvent.change(screen.getByPlaceholderText(/Paste your admin token here/i), {
      target: { value: "bad-token" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Sign In/i }));

    await waitFor(() => {
      expect(screen.getByText(/Login failed/i)).toBeInTheDocument();
    });
    expect(mockOnLogin).not.toHaveBeenCalled();
  });

  it("shows error toast on network error during submit", async () => {
    vi.mocked(post).mockRejectedValue(new Error("Network failure"));
    render(<LoginPage onLogin={mockOnLogin} />);

    fireEvent.change(screen.getByPlaceholderText(/Paste your admin token here/i), {
      target: { value: "any-token" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Sign In/i }));

    await waitFor(() => {
      expect(screen.getByText(/Login failed/i)).toBeInTheDocument();
    });
  });

  it("disables buttons during reveal and submit", async () => {
    vi.mocked(authApi.generateWebAdminToken).mockImplementation(
      () => new Promise((resolve) => setTimeout(() => resolve("token"), 100)),
    );
    render(<LoginPage onLogin={mockOnLogin} />);

    fireEvent.click(screen.getByRole("button", { name: /Reveal Token/i }));
    expect(screen.getByRole("button", { name: /Generating/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Sign In/i })).toBeDisabled();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
pnpm test:unit tests/components/auth/LoginPage.test.tsx
```

Expected: FAIL — `post` is not called in LoginPage yet, tests expecting `post` mock calls will fail.

- [ ] **Step 3: Refactor LoginPage to use web-client.post()**

In `src/components/auth/LoginPage.tsx`:

1. Replace the `fetch` import approach. Change:
```ts
import { authApi } from "@/lib/api";
import { setAuthToken } from "@/lib/api/web-client";
```
to:
```ts
import { authApi } from "@/lib/api";
import { post, setAuthToken } from "@/lib/api/web-client";
```

2. Replace the entire `handleSubmit` fetch block (lines 63-95) with:

```ts
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!token.trim()) {
      toast.error(
        t("login.tokenRequired", {
          defaultValue: "Please enter your admin token",
        }),
      );
      return;
    }

    setIsLoading(true);

    try {
      const { valid } = await post("/auth/verify", { token: token.trim() });

      if (!valid) {
        throw new Error("Invalid token");
      }

      setAuthToken(token.trim());
      toast.success(t("login.success", { defaultValue: "Login successful" }));
      onLogin();
    } catch (error) {
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
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
pnpm test:unit tests/components/auth/LoginPage.test.tsx
```

Expected: All 8 tests PASS.

- [ ] **Step 5: Run full unit test suite to prevent regressions**

```bash
pnpm test:unit
```

Expected: All existing tests still pass.

- [ ] **Step 6: Commit**

```bash
git add src/components/auth/LoginPage.tsx tests/components/auth/LoginPage.test.tsx
git commit -m "refactor: use web-client for LoginPage auth verify + add unit tests"
```

---

### Task 2: web-client.ts Unit Tests

**Files:**
- Create: `tests/lib/web-client.test.ts`
- Read-only reference: `src/lib/api/web-client.ts`

**Context:** `web-client.ts` exports `getAuthToken`, `setAuthToken`, `clearAuthToken`, `get`, `post`, `put`, `del`, `connectWebSocket`, `connectTerminalWebSocket`. Internal functions `fetchWithAuth` and `parseApiEnvelope` are tested indirectly through the public HTTP methods.

Key behaviors to verify:
- `authToken` persists to `localStorage`
- `fetch` receives `Authorization: Bearer <token>` when token is set
- 401 response → `clearAuthToken()` + redirect to `/login` (unless already on `/login`)
- Envelope parse: success, `success: false`, empty body, invalid JSON

- [ ] **Step 1: Write web-client tests**

Create `tests/lib/web-client.test.ts`:

```ts
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";

// We must re-import fresh each time because authToken is module-level state
async function importWebClient() {
  vi.resetModules();
  return import("@/lib/api/web-client");
}

const jsonResponse = (data: unknown, status = 200) =>
  new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json" },
  });

describe("web-client", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("auth token lifecycle", () => {
    it("sets token in memory and localStorage", async () => {
      const { setAuthToken, getAuthToken } = await importWebClient();
      setAuthToken("my-token");
      expect(getAuthToken()).toBe("my-token");
      expect(localStorage.getItem("cc_switch_token")).toBe("my-token");
    });

    it("clears token from memory and localStorage", async () => {
      const { setAuthToken, clearAuthToken, getAuthToken } = await importWebClient();
      setAuthToken("my-token");
      clearAuthToken();
      expect(getAuthToken()).toBeNull();
      expect(localStorage.getItem("cc_switch_token")).toBeNull();
    });

    it("restores token from localStorage on import", async () => {
      localStorage.setItem("cc_switch_token", "stored-token");
      const { getAuthToken } = await importWebClient();
      expect(getAuthToken()).toBe("stored-token");
    });
  });

  describe("fetchWithAuth", () => {
    it("injects Bearer token when authToken is set", async () => {
      const { get, setAuthToken } = await importWebClient();
      setAuthToken("bearer-token");
      const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: "ok" }),
      );

      await get("/test");

      expect(fetchMock).toHaveBeenCalledWith(
        "/api/v1/test",
        expect.objectContaining({
          headers: expect.objectContaining({
            Authorization: "Bearer bearer-token",
            "Content-Type": "application/json",
          }),
        }),
      );
    });

    it("omits Authorization header when no token is set", async () => {
      const { get } = await importWebClient();
      const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: "ok" }),
      );

      await get("/test");

      const callArgs = fetchMock.mock.calls[0][1] as RequestInit;
      const headers = callArgs.headers as Record<string, string>;
      expect(headers.Authorization).toBeUndefined();
    });

    it("clears token and redirects to /login on 401", async () => {
      const { get, setAuthToken, getAuthToken } = await importWebClient();
      setAuthToken("expired-token");

      // Mock window.location
      const originalLocation = window.location;
      // @ts-expect-error - mocking location for test
      delete window.location;
      window.location = { ...originalLocation, href: "/dashboard", pathname: "/dashboard" } as Location;

      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response(JSON.stringify({ success: false, error: "Unauthorized" }), { status: 401 }),
      );

      await expect(get("/test")).rejects.toThrow("Unauthorized");
      expect(getAuthToken()).toBeNull();
      expect(window.location.href).toBe("/login");

      // Restore location
      window.location = originalLocation;
    });

    it("does not redirect on 401 when already on /login", async () => {
      const { get, setAuthToken } = await importWebClient();
      setAuthToken("expired-token");

      const originalLocation = window.location;
      // @ts-expect-error - mocking location for test
      delete window.location;
      window.location = { ...originalLocation, href: "/login", pathname: "/login" } as Location;

      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response(JSON.stringify({ success: false, error: "Unauthorized" }), { status: 401 }),
      );

      await expect(get("/test")).rejects.toThrow("Unauthorized");
      expect(window.location.href).toBe("/login"); // unchanged

      window.location = originalLocation;
    });
  });

  describe("parseApiEnvelope", () => {
    it("returns data for a successful envelope", async () => {
      const { get } = await importWebClient();
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: { items: [1, 2] } }),
      );

      const result = await get("/items");
      expect(result).toEqual({ items: [1, 2] });
    });

    it("throws with error message when envelope success is false", async () => {
      const { get } = await importWebClient();
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: false, error: "Something went wrong", data: null }),
      );

      await expect(get("/items")).rejects.toThrow("Something went wrong");
    });

    it("throws with HTTP status for empty response body", async () => {
      const { get } = await importWebClient();
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("", { status: 500, statusText: "Internal Server Error" }),
      );

      await expect(get("/items")).rejects.toThrow("HTTP 500 Internal Server Error");
    });

    it("throws with HTTP status for invalid JSON", async () => {
      const { get } = await importWebClient();
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("not json", { status: 200 }),
      );

      await expect(get("/items")).rejects.toThrow("HTTP 200");
    });
  });

  describe("HTTP methods", () => {
    it("get uses GET method", async () => {
      const { get } = await importWebClient();
      const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: "get-result" }),
      );

      await get("/items");
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/v1/items",
        expect.objectContaining({ method: "GET" }),
      );
    });

    it("post uses POST method and serializes body", async () => {
      const { post } = await importWebClient();
      const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: "post-result" }),
      );

      await post("/items", { name: "test" });
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/v1/items",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ name: "test" }),
        }),
      );
    });

    it("put uses PUT method and serializes body", async () => {
      const { put } = await importWebClient();
      const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: "put-result" }),
      );

      await put("/items/1", { name: "updated" });
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/v1/items/1",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify({ name: "updated" }),
        }),
      );
    });

    it("del uses DELETE method", async () => {
      const { del } = await importWebClient();
      const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        jsonResponse({ success: true, data: "del-result" }),
      );

      await del("/items/1");
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/v1/items/1",
        expect.objectContaining({ method: "DELETE" }),
      );
    });
  });
});
```

- [ ] **Step 2: Run web-client tests**

```bash
pnpm test:unit tests/lib/web-client.test.ts
```

Expected: All 14 tests PASS.

- [ ] **Step 3: Run full unit test suite**

```bash
pnpm test:unit
```

Expected: All tests PASS (including new LoginPage tests from Task 1).

- [ ] **Step 4: Commit**

```bash
git add tests/lib/web-client.test.ts
git commit -m "test: add web-client unit tests for auth, 401, envelope parsing"
```

---

### Task 3: Playwright E2E Setup + Smoke Test

**Files:**
- Create: `playwright.config.ts`
- Create: `tests/e2e/web-smoke.spec.ts`
- Modify: `package.json`

**Context:** Playwright is already in `devDependencies` (`@playwright/test ^1.58.2`). We add a minimal config with `webServer` to auto-start the backend, plus one smoke test. The E2E test requires the debug Tauri binary at `./src-tauri/target/debug/cc-switch` — this must be built beforehand with `cargo build`.

- [ ] **Step 1: Create playwright.config.ts**

Create `playwright.config.ts`:

```ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: false, // Web server is shared
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Shared backend server
  reporter: "list",
  use: {
    baseURL: "http://localhost:13001",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "pnpm headless:debug:web",
    url: "http://localhost:13001/health",
    timeout: 60 * 1000,
    reuseExistingServer: !process.env.CI,
  },
});
```

- [ ] **Step 2: Add test:e2e script to package.json**

Add to `package.json` scripts section:
```json
"test:e2e": "playwright test",
"test:e2e:ui": "playwright test --ui"
```

- [ ] **Step 3: Write E2E smoke test**

Create `tests/e2e/web-smoke.spec.ts`:

```ts
import { test, expect } from "@playwright/test";

test.describe("web mode smoke", () => {
  test("login and navigate to dashboard", async ({ page }) => {
    // 1. Generate a token via the backend API
    const generateRes = await page.request.post("/api/v1/auth/generate", {});
    expect(generateRes.ok()).toBe(true);
    const generateBody = await generateRes.json();
    expect(generateBody.success).toBe(true);
    const token = generateBody.data as string;
    expect(token).toBeTruthy();

    // 2. Navigate to login page
    await page.goto("/");
    await expect(page.getByText("CC Switch")).toBeVisible();
    await expect(page.getByPlaceholder(/Paste your admin token here/i)).toBeVisible();

    // 3. Fill token and submit
    await page.fill('input[id="token"]', token);
    await page.click('button[type="submit"]');

    // 4. Wait for dashboard (no longer on login page)
    await expect(page).not.toHaveURL(/.*login.*/);

    // 5. Verify AppSwitcher tabs are visible
    await expect(page.getByText("Claude")).toBeVisible();
    await expect(page.getByText("Codex")).toBeVisible();
    await expect(page.getByText("Gemini")).toBeVisible();

    // 6. Click Claude tab and verify content loads
    await page.getByText("Claude").first().click();
    // The provider list or settings should appear
    await expect(page.getByText(/Provider|Settings|provider/i).first()).toBeVisible();
  });
});
```

- [ ] **Step 4: Ensure Playwright browsers are installed**

```bash
npx playwright install chromium
```

- [ ] **Step 5: Run E2E test (requires debug build)**

**Prerequisite:** The debug binary must exist. If not, build first:
```bash
cd src-tauri && cargo build && cd ..
```

Then run:
```bash
pnpm test:e2e
```

Expected: 1 test passes. If it fails, Playwright generates trace/screenshot in `test-results/`.

- [ ] **Step 6: Commit**

```bash
git add playwright.config.ts tests/e2e/web-smoke.spec.ts package.json
git commit -m "test: add Playwright E2E smoke test for web mode"
```

---

## Self-Review Checklist

### 1. Spec Coverage

| Spec Requirement | Plan Task |
|------------------|-----------|
| Refactor LoginPage to use `web-client.post()` | Task 1, Step 3 |
| `web-client` Bearer injection | Task 2, test "injects Bearer token" |
| `web-client` 401 → clear + redirect | Task 2, test "clears token and redirects" |
| `web-client` 401 on `/login` → no loop | Task 2, test "does not redirect on 401 when already on /login" |
| `web-client` envelope parse success | Task 2, test "returns data for a successful envelope" |
| `web-client` envelope parse error | Task 2, test "throws with error message when envelope success is false" |
| `web-client` envelope empty body | Task 2, test "throws with HTTP status for empty response body" |
| `web-client` envelope invalid JSON | Task 2, test "throws with HTTP status for invalid JSON" |
| `web-client` get/post/put/del | Task 2, HTTP methods section |
| LoginPage render | Task 1, test "renders the login form" |
| LoginPage empty submit | Task 1, test "shows error toast when submitting empty token" |
| LoginPage reveal token success | Task 1, test "reveals token and fills input on success" |
| LoginPage reveal token error | Task 1, test "shows error toast when reveal token fails" |
| LoginPage valid submit | Task 1, test "submits valid token, calls setAuthToken and onLogin" |
| LoginPage invalid token | Task 1, test "shows error toast when token is invalid" |
| LoginPage network error | Task 1, test "shows error toast on network error during submit" |
| LoginPage loading states | Task 1, test "disables buttons during reveal and submit" |
| E2E auto-start server | Task 3, `playwright.config.ts` `webServer` |
| E2E login → dashboard | Task 3, `web-smoke.spec.ts` |
| E2E tab switch | Task 3, `web-smoke.spec.ts` step 6 |

**Gap:** None. All spec requirements are mapped.

### 2. Placeholder Scan

- [x] No "TBD", "TODO", "implement later", "fill in details"
- [x] No vague instructions like "add appropriate error handling"
- [x] No "similar to Task N" references
- [x] Every step has exact file paths
- [x] Every test step has actual test code
- [x] Every implementation step has actual code

### 3. Type Consistency

- [x] `post` signature: `(url: string, body?: unknown)` — matches `src/lib/api/web-client.ts`
- [x] `/auth/verify` endpoint returns `{ valid: boolean }` — matches backend API
- [x] `/auth/generate` endpoint returns `ApiResponse<String>` — matches backend API
- [x] `setAuthToken` / `clearAuthToken` / `getAuthToken` — match exports from `web-client.ts`
- [x] `localStorage` key is `cc_switch_token` — matches `web-client.ts`

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-08-web-ui-testing.md`.**

Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
