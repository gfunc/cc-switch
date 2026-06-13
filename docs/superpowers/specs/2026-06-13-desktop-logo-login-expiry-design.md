# Desktop logo on login page + favicon, and fix login-expiry redirect

## Context

The web UI currently:
- Shows a hardcoded CSS "CC" circle on the login page instead of the desktop app icon.
- Has no favicon / web icon.
- On a 401/session expiry, swallows the error in the providers query and can briefly render an empty provider page before redirecting. After logging back in, the browser URL often stays on `/login` even though the providers page is shown.

## Goal

1. Use the existing desktop icon (`src-tauri/icons/icon.png`) as the login page logo.
2. Use `src/assets/icons/app-icon.png` as the favicon.
3. When the session expires, immediately show the login page and update the URL to `/login` — no empty provider page.
4. After a successful login, update the URL to `/` and refetch data cleanly.

## Chosen approach

**Option A — Lightweight URL sync.** Keep the current single-page architecture, avoid adding React Router, and fix the behavior with minimal changes:
- Import the desktop icon into the login page.
- Add a favicon link.
- Replace the full-page `window.location.href` redirect on 401 with an in-app event.
- Sync the URL with `window.history.replaceState` on auth state changes.
- Stop swallowing 401s in the providers query.

## 1. Assets & branding

### Login page logo

In `src/components/auth/LoginPage.tsx`, replace the gradient "CC" circle with an image:

```tsx
import logoSrc from "@tauri-icons/icon.png";

<img
  src={logoSrc}
  alt={t("login.logoAlt", { defaultValue: "CC Switch" })}
  className="w-16 h-16 rounded-full object-cover shadow-lg"
/>
```

To make `src-tauri/icons/icon.png` importable from the frontend without duplicating it, add a Vite/TypeScript alias:

- `@tauri-icons` → `<projectRoot>/src-tauri/icons`

This keeps the desktop icon as the single source of truth.

### Favicon / web icon

In `src/index.html`, add:

```html
<link rel="icon" type="image/png" href="/assets/icons/app-icon.png" />
```

`src/assets/icons/app-icon.png` is copied to `dist/assets/icons/app-icon.png` at build time. The production Axum server already serves `/assets/*` from `dist/assets`, and the Vite dev server serves from `src/assets`, so the absolute path works in both dev and production.

## 2. Auth state & URL sync

### Stop the full-page reload on 401

In `src/lib/api/web-client.ts`, replace the `window.location.href = "/login"` redirect with a custom event:

```ts
window.dispatchEvent(new CustomEvent("auth:expired"));
```

`clearAuthToken()` is still called so the token is removed from memory and `localStorage`.

### React to the event in `App.tsx`

Add a `useEffect` that listens for `auth:expired`:

1. Call `clearAuthToken()`.
2. Clear the React Query cache with `queryClient.clear()` so stale/empty data is removed.
3. Set `isAuthenticated` to `false`.
4. Sync the URL to `/login` with `window.history.replaceState`.

### Fix the URL after login

In `App.tsx`'s `handleLogin`:

1. Set `isAuthenticated` to `true`.
2. Clear the React Query cache so providers/settings refetch with the new token instead of displaying cached empty data.
3. Sync the URL to `/` with `window.history.replaceState`.

### Initial-load consistency

On first render, if the user is not authenticated and the URL is not already `/login`, replace the URL to `/login` so the address bar matches the login UI.

## 3. Query behavior on 401

In `src/lib/query/queries.ts`, the providers query currently catches errors from `providersApi.getAll()` and `providersApi.getCurrent()` and returns `{ providers: {}, currentProviderId: null }`. The UI treats that as "no providers configured," producing the empty provider page.

Fix:

- Remove the `try/catch` wrappers in the `useProvidersQuery` query function.
- Let the query throw on 401 so React Query sees an error state.
- `App.tsx` handles the `auth:expired` event immediately, so the providers component unmounts before an error or empty list is visible.
- Optionally guard `useSettingsQuery` (called above the auth gate) with `enabled: isAuthenticated` so it does not fire while the user is logged out.

## 4. Files expected to change

- `src/components/auth/LoginPage.tsx` — use desktop icon image.
- `src/index.html` — add favicon link.
- `vite.config.ts` — add `@tauri-icons` alias.
- `tsconfig.json` / `tsconfig.app.json` — add `@tauri-icons` path alias.
- `src/lib/api/web-client.ts` — dispatch `auth:expired` instead of redirecting.
- `src/App.tsx` — listen for `auth:expired`, sync URL, clear cache on login/logout.
- `src/lib/query/queries.ts` — stop swallowing errors in `useProvidersQuery`.
- `tests/lib/web-client.test.ts` — update 401 redirect assertions.

## 5. Testing & verification

### Unit / integration tests

- Update `tests/lib/web-client.test.ts` to assert that a 401 dispatches `auth:expired` instead of setting `window.location.href`.
- Add coverage for `handleLogin` clearing the query cache and rewriting the URL to `/`.

### Manual checks

1. Open `/login` while unauthenticated → login page shows the desktop logo and the browser tab displays `app-icon.png`.
2. Sign in → URL changes to `/` and providers load (not the empty state).
3. Clear the `cc_switch_token` from `localStorage` or wait for expiry → app switches to the login page, URL is `/login`, and no empty providers page is visible.
4. Refresh on `/login` while unauthenticated → still shows the login page (backend fallback serves `index.html`).

### Build / lint

Run the project's build and lint commands and confirm they pass before marking the work complete.
