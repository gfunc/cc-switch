# Desktop logo on login page + favicon, and fix login-expiry redirect

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the desktop app icon to the login page and `app-icon.png` as the favicon, and fix the auth-expiry flow so the URL correctly tracks `/login` ↔ `/` and the empty provider page never appears.

**Architecture:** Keep the existing single-page architecture (no React Router). Add a small `useWebAuthSync` hook that listens for an `auth:expired` event from the API client, clears the React Query cache, flips auth state, and syncs the URL via `history.replaceState`. Update the API client to dispatch that event on 401 instead of doing a full-page redirect. Swap the CSS logo for an image import and add a favicon link.

**Tech Stack:** React, TypeScript, Vite, Vitest, React Query, Tailwind, jsdom.

---

## File structure

- `src/index.html` — add favicon `<link>`.
- `vite.config.ts` — add `@tauri-icons` alias pointing at `src-tauri/icons`.
- `tsconfig.json` — add `@tauri-icons/*` path mapping.
- `src/components/auth/LoginPage.tsx` — render desktop icon image.
- `tests/components/auth/LoginPage.test.tsx` — assert logo image renders.
- `src/lib/api/web-client.ts` — dispatch `auth:expired` on 401.
- `tests/lib/web-client.test.ts` — update 401 assertions.
- `src/hooks/useWebAuthSync.ts` — new hook: event listener + URL sync + cache clear.
- `tests/hooks/useWebAuthSync.test.tsx` — unit tests for the hook.
- `src/App.tsx` — wire in `useWebAuthSync`, clear cache on login, guard settings query.
- `src/lib/query/queries.ts` — stop swallowing errors in `useProvidersQuery`; add optional `enabled` to `useSettingsQuery`.

---

## Task 1: Add favicon link to `src/index.html`

**Files:**
- Modify: `src/index.html:7`

- [ ] **Step 1: Add the favicon link inside `<head>`**

```html
<link rel="icon" type="image/png" href="/assets/icons/app-icon.png" />
```

The full `<head>` should look like:

```html
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Claude Code 供应商切换器</title>
  <link rel="icon" type="image/png" href="/assets/icons/app-icon.png" />
</head>
```

- [ ] **Step 2: Verify the link is present in the production build output**

Run:

```bash
pnpm build:renderer
```

Expected: build succeeds.

Then run:

```bash
grep -F 'app-icon.png' dist/index.html
```

Expected output contains `app-icon.png`.

- [ ] **Step 3: Commit**

```bash
git add src/index.html
git commit -m "feat(web): add app-icon.png favicon"
```

---

## Task 2: Make `src-tauri/icons` importable from the frontend

**Files:**
- Modify: `vite.config.ts:24-27`
- Modify: `tsconfig.json:18-20`

- [ ] **Step 1: Add Vite alias**

In `vite.config.ts`, update the `resolve.alias` object:

```ts
resolve: {
  alias: {
    "@": path.resolve(__dirname, "./src"),
    "@tauri-icons": path.resolve(__dirname, "./src-tauri/icons"),
  },
},
```

- [ ] **Step 2: Add TypeScript path mapping**

In `tsconfig.json`, update the `paths` object:

```json
"paths": {
  "@/*": ["src/*"],
  "@tauri-icons/*": ["src-tauri/icons/*"]
}
```

- [ ] **Step 3: Verify typecheck passes**

Run:

```bash
pnpm typecheck
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add vite.config.ts tsconfig.json
git commit -m "chore(web): add @tauri-icons alias for desktop icon assets"
```

---

## Task 3: Use the desktop icon on the login page

**Files:**
- Modify: `src/components/auth/LoginPage.tsx:1-19`, `src/components/auth/LoginPage.tsx:107-113`
- Modify: `tests/components/auth/LoginPage.test.tsx`

- [ ] **Step 1: Write the failing test**

Add this test to `tests/components/auth/LoginPage.test.tsx`:

```tsx
it("renders the desktop logo image", () => {
  renderLoginPage();
  const logo = screen.getByRole("img", { name: "login.logoAlt" });
  expect(logo).toBeInTheDocument();
  expect(logo.getAttribute("src")).toContain("icon");
});
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
pnpm vitest run tests/components/auth/LoginPage.test.tsx
```

Expected: FAIL — `Unable to find role="img"`.

- [ ] **Step 3: Replace the CSS logo with an image**

In `src/components/auth/LoginPage.tsx`, add the import near the top:

```tsx
import logoSrc from "@tauri-icons/icon.png";
```

Replace the logo markup:

```tsx
<div className="flex justify-center mb-4">
  <img
    src={logoSrc}
    alt={t("login.logoAlt", { defaultValue: "CC Switch" })}
    className="w-16 h-16 rounded-full object-cover shadow-lg"
  />
</div>
```

Remove the old gradient circle markup.

- [ ] **Step 4: Run the test and confirm it passes**

Run:

```bash
pnpm vitest run tests/components/auth/LoginPage.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/auth/LoginPage.tsx tests/components/auth/LoginPage.test.tsx
git commit -m "feat(web): use desktop icon on login page"
```

---

## Task 4: Dispatch `auth:expired` from the API client on 401

**Files:**
- Modify: `src/lib/api/web-client.ts:68-75`
- Modify: `tests/lib/web-client.test.ts:82-134`

- [ ] **Step 1: Update the 401 test to expect an event dispatch**

Replace the existing test `"clears token and redirects to /login on 401"` in `tests/lib/web-client.test.ts` with:

```tsx
it("clears token and dispatches auth:expired on 401", async () => {
  const { setAuthToken, get, getAuthToken } = await importWebClient();
  setAuthToken("expired-token");

  const dispatchSpy = vi.spyOn(window, "dispatchEvent");

  vi.spyOn(globalThis, "fetch").mockResolvedValue(
    new Response(JSON.stringify({ success: false, error: "Unauthorized" }), {
      status: 401,
    }),
  );

  await expect(get("/test")).rejects.toThrow("Unauthorized");
  expect(getAuthToken()).toBeNull();
  expect(localStorage.getItem("cc_switch_token")).toBeNull();
  expect(dispatchSpy).toHaveBeenCalledWith(
    expect.objectContaining({ type: "auth:expired" }),
  );
});
```

Replace the test `"does not redirect on 401 when already on /login"` with:

```tsx
it("still dispatches auth:expired on 401 when already on /login", async () => {
  const { setAuthToken, get } = await importWebClient();
  setAuthToken("expired-token");

  const dispatchSpy = vi.spyOn(window, "dispatchEvent");

  const originalLocation = window.location;
  Object.defineProperty(window, "location", {
    writable: true,
    value: { ...originalLocation, pathname: "/login", href: "/login" },
  });

  vi.spyOn(globalThis, "fetch").mockResolvedValue(
    new Response(JSON.stringify({ success: false, error: "Unauthorized" }), {
      status: 401,
    }),
  );

  try {
    await expect(get("/test")).rejects.toThrow("Unauthorized");
    expect(dispatchSpy).toHaveBeenCalledWith(
      expect.objectContaining({ type: "auth:expired" }),
    );
  } finally {
    Object.defineProperty(window, "location", {
      writable: true,
      value: originalLocation,
    });
  }
});
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run:

```bash
pnpm vitest run tests/lib/web-client.test.ts
```

Expected: two FAILs because the event is not dispatched yet.

- [ ] **Step 3: Replace the redirect with an event dispatch**

In `src/lib/api/web-client.ts`, replace:

```ts
if (response.status === 401) {
  clearAuthToken();
  // Only redirect if not already on login page to prevent redirect loops
  if (!window.location.pathname.includes('/login')) {
    window.location.href = "/login";
  }
  throw new Error("Unauthorized");
}
```

with:

```ts
if (response.status === 401) {
  clearAuthToken();
  window.dispatchEvent(new CustomEvent("auth:expired"));
  throw new Error("Unauthorized");
}
```

- [ ] **Step 4: Run the tests and confirm they pass**

Run:

```bash
pnpm vitest run tests/lib/web-client.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/api/web-client.ts tests/lib/web-client.test.ts
git commit -m "feat(web): dispatch auth:expired event on 401 instead of redirecting"
```

---

## Task 5: Add `useWebAuthSync` hook

**Files:**
- Create: `src/hooks/useWebAuthSync.ts`
- Create: `tests/hooks/useWebAuthSync.test.tsx`

- [ ] **Step 1: Write the failing tests**

Create `tests/hooks/useWebAuthSync.test.tsx`:

```tsx
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

const clearAuthTokenMock = vi.fn();

vi.mock("@/lib/api/web-client", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api/web-client")>();
  return {
    ...actual,
    clearAuthToken: clearAuthTokenMock,
  };
});

vi.mock("@/lib/environment", () => ({
  isTauri: () => false,
  isLinux: () => false,
  isMac: () => false,
  isWindows: () => false,
}));

import { useWebAuthSync } from "@/hooks/useWebAuthSync";

function TestHarness({
  initialAuth = false,
  client,
}: {
  initialAuth?: boolean;
  client: QueryClient;
}) {
  const [auth, setAuth] = useState(initialAuth);
  useWebAuthSync(auth, setAuth);
  return <div data-testid="auth-state">{auth ? "in" : "out"}</div>;
}

function renderHarness(initialAuth = false) {
  const client = new QueryClient();
  return {
    client,
    ...render(
      <QueryClientProvider client={client}>
        <TestHarness initialAuth={initialAuth} client={client} />
      </QueryClientProvider>,
    ),
  };
}

describe("useWebAuthSync", () => {
  beforeEach(() => {
    window.history.replaceState({}, "", "/");
    clearAuthTokenMock.mockClear();
  });

  afterEach(() => {
    window.history.replaceState({}, "", "/");
  });

  it("syncs URL to /login when unauthenticated", () => {
    renderHarness(false);
    expect(window.location.pathname).toBe("/login");
    expect(screen.getByTestId("auth-state")).toHaveTextContent("out");
  });

  it("syncs URL to / when authenticated", () => {
    window.history.replaceState({}, "", "/login");
    renderHarness(true);
    expect(window.location.pathname).toBe("/");
    expect(screen.getByTestId("auth-state")).toHaveTextContent("in");
  });

  it("handles auth:expired by clearing auth, cache, and URL", async () => {
    const { client } = renderHarness(true);
    client.setQueryData(["test"], "value");

    act(() => {
      window.dispatchEvent(new CustomEvent("auth:expired"));
    });

    await waitFor(() =>
      expect(screen.getByTestId("auth-state")).toHaveTextContent("out"),
    );
    expect(window.location.pathname).toBe("/login");
    expect(clearAuthTokenMock).toHaveBeenCalledTimes(1);
    expect(client.getQueryData(["test"])).toBeUndefined();
  });
});
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run:

```bash
pnpm vitest run tests/hooks/useWebAuthSync.test.tsx
```

Expected: FAIL — `useWebAuthSync` module not found.

- [ ] **Step 3: Implement the hook**

Create `src/hooks/useWebAuthSync.ts`:

```ts
import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { clearAuthToken } from "@/lib/api/web-client";
import { isTauri } from "@/lib/environment";

function syncAuthUrl(isAuthenticated: boolean) {
  if (isTauri()) return;

  const pathname = window.location.pathname;
  if (isAuthenticated && pathname === "/login") {
    window.history.replaceState({}, "", "/");
  } else if (!isAuthenticated && pathname !== "/login") {
    window.history.replaceState({}, "", "/login");
  }
}

export function useWebAuthSync(
  isAuthenticated: boolean,
  setIsAuthenticated: (value: boolean) => void,
) {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (isTauri()) return;

    syncAuthUrl(isAuthenticated);

    const handleExpired = () => {
      if (!isAuthenticated) return;
      clearAuthToken();
      queryClient.clear();
      setIsAuthenticated(false);
      syncAuthUrl(false);
    };

    window.addEventListener("auth:expired", handleExpired);
    return () => window.removeEventListener("auth:expired", handleExpired);
  }, [isAuthenticated, queryClient, setIsAuthenticated]);
}
```

- [ ] **Step 4: Run the tests and confirm they pass**

Run:

```bash
pnpm vitest run tests/hooks/useWebAuthSync.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/hooks/useWebAuthSync.ts tests/hooks/useWebAuthSync.test.tsx
git commit -m "feat(web): add useWebAuthSync hook for URL and cache sync"
```

---

## Task 6: Wire `useWebAuthSync` into `App.tsx`

**Files:**
- Modify: `src/App.tsx` imports, `src/App.tsx:179-187`, `src/App.tsx:201`

- [ ] **Step 1: Update imports in `src/App.tsx`**

Change:

```ts
import { getAuthToken } from "@/lib/api/web-client";
```

to:

```ts
import { clearAuthToken, getAuthToken } from "@/lib/api/web-client";
```

Add:

```ts
import { useWebAuthSync } from "@/hooks/useWebAuthSync";
```

- [ ] **Step 2: Call the hook and update login/logout handlers**

Inside `App()`, after `const queryClient = useQueryClient();`, add:

```ts
useWebAuthSync(isAuthenticated, setIsAuthenticated);
```

Update `handleLogin`:

```ts
const handleLogin = () => {
  setIsAuthenticated(true);
  // Cached empty/401 data from before login must be removed so queries refetch
  // with the new token instead of briefly showing the empty provider page.
  queryClient.clear();
};
```

Update the settings query call:

```ts
const { data: settingsData } = useSettingsQuery({ enabled: isAuthenticated });
```

- [ ] **Step 3: Run App integration tests**

Run:

```bash
pnpm vitest run tests/integration/App.test.tsx
```

Expected: PASS. These tests run in mocked Tauri mode, so the new web-only hook should be a no-op.

- [ ] **Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "feat(web): wire useWebAuthSync and clear cache on login"
```

---

## Task 7: Stop swallowing provider-query errors

**Files:**
- Modify: `src/lib/query/queries.ts:54-95`

- [ ] **Step 1: Remove error swallowing in `useProvidersQuery`**

Replace the `queryFn` in `useProvidersQuery` with:

```ts
queryFn: async () => {
  const providers = await providersApi.getAll(appId);
  const currentProviderId = await providersApi.getCurrent(appId);

  return {
    providers: sortProviders(providers),
    currentProviderId,
  };
},
```

Remove the two `try/catch` blocks and the `let` declarations that initialized empty defaults.

- [ ] **Step 2: Add optional `enabled` to `useSettingsQuery`**

Change:

```ts
export const useSettingsQuery = (): UseQueryResult<Settings> => {
  return useQuery({
    queryKey: ["settings"],
    queryFn: async () => settingsApi.get(),
  });
};
```

to:

```ts
export const useSettingsQuery = (
  options?: { enabled?: boolean },
): UseQueryResult<Settings> => {
  const { enabled = true } = options || {};
  return useQuery({
    queryKey: ["settings"],
    queryFn: async () => settingsApi.get(),
    enabled,
  });
};
```

- [ ] **Step 3: Run typecheck and unit tests**

Run:

```bash
pnpm typecheck
```

Expected: no errors.

Run:

```bash
pnpm test:unit
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/query/queries.ts
git commit -m "feat(web): surface provider query errors and guard settings query by auth"
```

---

## Task 8: Final verification

- [ ] **Step 1: Format check**

Run:

```bash
pnpm format:check
```

Expected: no formatting issues.

- [ ] **Step 2: Full unit test suite**

Run:

```bash
pnpm test:unit
```

Expected: PASS.

- [ ] **Step 3: Production renderer build**

Run:

```bash
pnpm build:renderer
```

Expected: build succeeds and `dist/index.html` contains the favicon link and a generated asset reference for the login logo.

- [ ] **Step 4: Manual smoke test (if running the web server)**

1. Start the web server or run `pnpm dev:renderer` against a backend with a valid token endpoint.
2. Open `/login` while unauthenticated → desktop logo visible; tab shows favicon.
3. Log in → URL changes to `/` and providers appear.
4. Delete `cc_switch_token` from `localStorage` or trigger a 401 → app switches back to login with URL `/login` and no empty provider page.

- [ ] **Step 5: Final commit if any changes remain**

```bash
git status
# add anything not yet committed, then:
git commit -m "feat(web): desktop login logo, favicon, and auth-expiry redirect fix"
```

---

## Plan self-review

**Spec coverage:**
- Login page logo using `src-tauri/icons/icon.png` → Task 3.
- Favicon using `src/assets/icons/app-icon.png` → Task 1.
- Show login page on expiry, URL `/login` → Tasks 4, 5, 6.
- URL `/` after login → Tasks 5, 6.
- Stop empty provider page → Tasks 4, 5, 7.

**Placeholder scan:** No TBDs or vague steps. Every task has exact file paths, code, and commands.

**Type consistency:** `useSettingsQuery` accepts optional `{ enabled?: boolean }` everywhere it is called without arguments remains valid. `useWebAuthSync` signature is consistent across implementation and tests. `auth:expired` event name is consistent across API client, hook, and tests.
