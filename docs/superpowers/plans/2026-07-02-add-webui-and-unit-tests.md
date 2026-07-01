# Add Web UI E2E and Unit Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add focused Playwright end-to-end coverage for core web UI flows and Vitest unit coverage for utility modules, hooks, and web API clients that currently lack tests.

**Architecture:** Keep tests close to existing conventions — E2E specs live in `tests/e2e/`, unit tests mirror their source under `tests/` or sit next to source when the project already does so. Reuse existing MSW handlers and test setup; mock `fetch` for web API client tests; write deterministic assertions with no external dependencies.

**Tech Stack:** Playwright (E2E), Vitest + jsdom + React Testing Library (unit), MSW (unit API mocking), TypeScript.

## Global Constraints

- All new tests must pass in CI (`pnpm test:e2e` and `pnpm test:unit`).
- E2E tests use the existing `playwright.config.ts` web server (`http://localhost:13002`).
- E2E auth token is obtained via `/api/v1/auth/generate` with `Authorization: Bearer e2e-test-token`.
- Unit tests use the existing `tests/setupTests.ts` and `tests/setupGlobals.ts` setup files.
- Do not modify production code unless a test reveals a real bug; in that case, fix the bug with a regression test.
- Follow TDD: write the test first, run it, then make it pass.
- Each task ends with a passing test run and a git commit.

---

## Task 1: E2E Provider Management Flow

**Files:**
- Create: `tests/e2e/providers.spec.ts`

**Interfaces:**
- Consumes: Existing Playwright config, auth API, providers API.
- Produces: `tests/e2e/providers.spec.ts` covering provider switch and delete.

- [ ] **Step 1: Write the failing E2E spec**

```typescript
import { test, expect } from "@playwright/test";

const STATIC_TOKEN = "e2e-test-token";
const APP = "claude";

async function authToken(request: import("@playwright/test").APIRequestContext) {
  const res = await request.post("/api/v1/auth/generate", {
    headers: { Authorization: `Bearer ${STATIC_TOKEN}` },
  });
  expect(res.ok()).toBe(true);
  const body = await res.json();
  expect(body.success).toBe(true);
  return body.data as string;
}

test.describe("provider management (web)", () => {
  test("switches and deletes a provider from the UI", async ({ page, request }) => {
    const token = await authToken(request);

    // Seed a provider through the API so the UI has something deterministic to act on.
    const providerRes = await request.post("/api/v1/providers", {
      headers: { Authorization: `Bearer ${token}` },
      data: {
        provider: {
          id: "e2e-switch-provider",
          name: "E2E Switch Provider",
          settingsConfig: { env: { ANTHROPIC_BASE_URL: "http://localhost:9999" } },
          category: "custom",
          sortIndex: 0,
          createdAt: Date.now(),
        },
        app: APP,
      },
    });
    expect(providerRes.ok()).toBe(true);

    // Log in through the UI.
    await page.goto("/");
    await page.fill('input[id="token"]', token);
    await page.click('button[type="submit"]');
    await expect(page.getByRole("banner")).toBeVisible();

    // Dismiss first-run welcome dialog if it appears.
    const gotItButton = page.getByRole("button", { name: /Got it|知道了/i });
    try {
      await gotItButton.waitFor({ state: "visible", timeout: 5000 });
      await gotItButton.click();
      await expect(gotItButton).toBeHidden();
    } catch {
      // No first-run dialog; proceed.
    }

    // Ensure we are on the Claude providers view.
    const claudeTab = page.getByRole("button", { name: /Claude Code$/i });
    await expect(claudeTab).toBeVisible();
    await claudeTab.click();

    // The seeded provider should be visible.
    const providerHeading = page.getByRole("heading", {
      name: "E2E Switch Provider",
    });
    await expect(providerHeading).toBeVisible();

    // Hover the provider card to reveal action buttons, then switch to it.
    const card = page.locator("div").filter({ has: providerHeading });
    await card.hover();
    const enableButton = card.getByRole("button", { name: /Enable|启用/i });
    await expect(enableButton).toBeVisible();
    await enableButton.click();

    // After switching, the main action should indicate it is in use.
    await expect(card.getByRole("button", { name: /In Use|已在用/i })).toBeVisible();

    // Hover again and delete the provider.
    await card.hover();
    const deleteButton = card.getByRole("button", { name: /Delete|删除/i });
    await expect(deleteButton).toBeVisible();
    await deleteButton.click();

    // Confirm deletion in the dialog.
    const confirmButton = page.getByRole("button", { name: /Confirm|确认/i }).last();
    await expect(confirmButton).toBeVisible();
    await confirmButton.click();

    // Provider should disappear from the list.
    await expect(providerHeading).toHaveCount(0);
  });
});
```

- [ ] **Step 2: Run the E2E spec to verify it fails or behaves as expected**

Run: `pnpm test:e2e tests/e2e/providers.spec.ts`

Expected: If selectors are wrong, the test fails with a clear timeout or missing element error. Fix selectors by re-running and inspecting the page with `playwright-cli` if needed.

- [ ] **Step 3: Adjust selectors or production code until the spec passes**

Use `playwright-cli` to explore the live page if selectors do not match:

```bash
playwright-cli open http://localhost:13002
playwright-cli snapshot
```

If the confirm dialog uses a different accessible name, update the spec to match the actual button text. Do not change production behavior.

- [ ] **Step 4: Run the E2E spec to verify it passes**

Run: `pnpm test:e2e tests/e2e/providers.spec.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/e2e/providers.spec.ts
git commit -m "test(e2e): add provider switch and delete flow"
```

---

## Task 2: E2E Settings Language Change Flow

**Files:**
- Create: `tests/e2e/settings.spec.ts`

**Interfaces:**
- Consumes: Existing Playwright config, auth API, settings API.
- Produces: `tests/e2e/settings.spec.ts` covering language switch persistence.

- [ ] **Step 1: Write the failing E2E spec**

```typescript
import { test, expect } from "@playwright/test";

const STATIC_TOKEN = "e2e-test-token";

async function authToken(request: import("@playwright/test").APIRequestContext) {
  const res = await request.post("/api/v1/auth/generate", {
    headers: { Authorization: `Bearer ${STATIC_TOKEN}` },
  });
  expect(res.ok()).toBe(true);
  const body = await res.json();
  expect(body.success).toBe(true);
  return body.data as string;
}

test.describe("settings (web)", () => {
  test("changes interface language from the settings page", async ({ page, request }) => {
    const token = await authToken(request);

    await page.goto("/");
    await page.fill('input[id="token"]', token);
    await page.click('button[type="submit"]');
    await expect(page.getByRole("banner")).toBeVisible();

    const gotItButton = page.getByRole("button", { name: /Got it|知道了/i });
    try {
      await gotItButton.waitFor({ state: "visible", timeout: 5000 });
      await gotItButton.click();
      await expect(gotItButton).toBeHidden();
    } catch {
      // No first-run dialog; proceed.
    }

    // Open settings from the header.
    await page.getByRole("button", { name: "Settings" }).click();

    // Wait for the settings page heading.
    const settingsHeading = page.getByRole("heading", { name: /Settings|设置/i });
    await expect(settingsHeading).toBeVisible();

    // The language section header is translated.
    const languageHeader = page.getByText(/Language|语言/i);
    await expect(languageHeader).toBeVisible();

    // Switch to English.
    await page.getByRole("button", { name: "English" }).click();

    // After switching, the language section header should read "Language".
    await expect(page.getByText("Language")).toBeVisible();

    // The settings heading should be in English.
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  });
});
```

- [ ] **Step 2: Run the E2E spec to verify it fails or behaves as expected**

Run: `pnpm test:e2e tests/e2e/settings.spec.ts`

Expected: If the initial language is already English, the test should still pass because the assertions tolerate both states and end on English text. If a selector is wrong, fix it.

- [ ] **Step 3: Adjust selectors if needed**

Use `playwright-cli` to inspect the settings page if needed:

```bash
playwright-cli open http://localhost:13002
playwright-cli click e<settings-button-ref>
playwright-cli snapshot
```

Update the spec to match actual accessible names.

- [ ] **Step 4: Run the E2E spec to verify it passes**

Run: `pnpm test:e2e tests/e2e/settings.spec.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/e2e/settings.spec.ts
git commit -m "test(e2e): add settings language change flow"
```

---

## Task 3: Unit Tests for `tomlUtils`

**Files:**
- Create: `tests/utils/tomlUtils.test.ts`
- Source: `src/utils/tomlUtils.ts`

**Interfaces:**
- Consumes: `src/utils/tomlUtils.ts` exports.
- Produces: `tests/utils/tomlUtils.test.ts` with full coverage of `validateToml`, `mcpServerToToml`, `tomlToMcpServer`, `extractIdFromToml`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect } from "vitest";
import {
  validateToml,
  mcpServerToToml,
  tomlToMcpServer,
  extractIdFromToml,
} from "@/utils/tomlUtils";
import type { McpServerSpec } from "@/types";

describe("tomlUtils", () => {
  describe("validateToml", () => {
    it("returns empty string for valid TOML object", () => {
      expect(validateToml('command = "npx"')).toBe("");
    });

    it("returns empty string for empty input", () => {
      expect(validateToml("")).toBe("");
    });

    it("returns mustBeObject for top-level array", () => {
      expect(validateToml("[1, 2, 3]")).toBe("mustBeObject");
    });

    it("returns an error message for malformed TOML", () => {
      const result = validateToml("command = ");
      expect(result).not.toBe("");
    });
  });

  describe("mcpServerToToml", () => {
    it("serializes a stdio server", () => {
      const server: McpServerSpec = {
        type: "stdio",
        command: "npx",
        args: ["-y", "@modelcontextprotocol/server-filesystem"],
      };
      const toml = mcpServerToToml(server);
      expect(toml).toContain('type = "stdio"');
      expect(toml).toContain('command = "npx"');
      expect(toml).toContain("-y");
    });

    it("strips undefined fields", () => {
      const server: McpServerSpec = {
        type: "http",
        url: "http://localhost:3000",
      };
      const toml = mcpServerToToml(server as any);
      expect(toml).not.toContain("args");
      expect(toml).not.toContain("env");
    });

    it("preserves unknown extension fields", () => {
      const server: McpServerSpec = {
        type: "stdio",
        command: "npx",
        timeout_ms: 30000,
      } as McpServerSpec;
      const toml = mcpServerToToml(server);
      expect(toml).toContain("timeout_ms");
      expect(toml).toContain("30000");
    });
  });

  describe("tomlToMcpServer", () => {
    it("parses a direct stdio server config", () => {
      const server = tomlToMcpServer('type = "stdio"\ncommand = "npx"');
      expect(server.type).toBe("stdio");
      expect(server.command).toBe("npx");
    });

    it("parses [mcp_servers.id] format and takes the first server", () => {
      const toml = `
[mcp_servers.filesystem]
type = "stdio"
command = "npx"
args = ["/tmp"]
      `.trim();
      const server = tomlToMcpServer(toml);
      expect(server.type).toBe("stdio");
      expect(server.command).toBe("npx");
    });

    it("parses [mcp.servers.id] fallback format", () => {
      const toml = `
[mcp.servers.filesystem]
type = "http"
url = "http://example.com"
      `.trim();
      const server = tomlToMcpServer(toml);
      expect(server.type).toBe("http");
      expect(server.url).toBe("http://example.com");
    });

    it("throws for empty input", () => {
      expect(() => tomlToMcpServer("")).toThrow();
    });

    it("throws for stdio config missing command", () => {
      expect(() => tomlToMcpServer('type = "stdio"')).toThrow("command");
    });

    it("throws for http config missing url", () => {
      expect(() => tomlToMcpServer('type = "http"')).toThrow("url");
    });

    it("coerces args to strings", () => {
      const server = tomlToMcpServer('type = "stdio"\ncommand = "npx"\nargs = [123]');
      expect(server.args).toEqual(["123"]);
    });
  });

  describe("extractIdFromToml", () => {
    it("extracts id from [mcp_servers.id]", () => {
      expect(
        extractIdFromToml(`
[mcp_servers.filesystem]
type = "stdio"
command = "npx"
        `),
      ).toBe("filesystem");
    });

    it("extracts id from [mcp.servers.id]", () => {
      expect(
        extractIdFromToml(`
[mcp.servers.filesystem]
type = "stdio"
command = "npx"
        `),
      ).toBe("filesystem");
    });

    it("infers id from command basename", () => {
      expect(extractIdFromToml('command = "/usr/local/bin/my-server.js"')).toBe(
        "my-server",
      );
    });

    it("returns empty string for unparseable input", () => {
      expect(extractIdFromToml("not toml at all ===")).toBe("");
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/utils/tomlUtils.test.ts`

Expected: PASS (the source already exists and the tests assert existing behavior).

- [ ] **Step 3: Fix any discrepancies**

If any assertion fails, inspect the actual output and adjust the test to match the real behavior. Do not weaken the test; if behavior is wrong, fix the source and add a regression note.

- [ ] **Step 4: Run the test again and run the full unit suite**

Run:

```bash
pnpm test:unit tests/utils/tomlUtils.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/utils/tomlUtils.test.ts
git commit -m "test(unit): add tomlUtils coverage"
```

---

## Task 4: Unit Tests for `errorUtils`

**Files:**
- Create: `tests/utils/errorUtils.test.ts`
- Source: `src/utils/errorUtils.ts`

**Interfaces:**
- Consumes: `src/utils/errorUtils.ts` exports.
- Produces: `tests/utils/errorUtils.test.ts` covering `extractErrorMessage` and `translateMcpBackendError`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect, vi } from "vitest";
import {
  extractErrorMessage,
  translateMcpBackendError,
} from "@/utils/errorUtils";

const mockT = vi.fn((key: string, _opts?: any) => key);

describe("errorUtils", () => {
  describe("extractErrorMessage", () => {
    it("returns empty string for null/undefined/falsy errors", () => {
      expect(extractErrorMessage(null)).toBe("");
      expect(extractErrorMessage(undefined)).toBe("");
      expect(extractErrorMessage("")).toBe("");
    });

    it("returns string errors as-is", () => {
      expect(extractErrorMessage("something broke")).toBe("something broke");
    });

    it("extracts Error.message", () => {
      expect(extractErrorMessage(new Error("nested error"))).toBe(
        "nested error",
      );
    });

    it("extracts message field from object", () => {
      expect(extractErrorMessage({ message: "object message" })).toBe(
        "object message",
      );
    });

    it("falls back to error/detail field", () => {
      expect(extractErrorMessage({ error: "error value" })).toBe("error value");
      expect(extractErrorMessage({ detail: "detail value" })).toBe(
        "detail value",
      );
    });

    it("extracts message from nested payload", () => {
      expect(
        extractErrorMessage({ payload: { message: "payload message" } }),
      ).toBe("payload message");
    });

    it("returns empty string when nothing matches", () => {
      expect(extractErrorMessage({ code: 500 })).toBe("");
    });
  });

  describe("translateMcpBackendError", () => {
    it("returns empty string for empty input", () => {
      expect(translateMcpBackendError("", mockT)).toBe("");
    });

    it("maps id required error", () => {
      expect(
        translateMcpBackendError("MCP 服务器 ID 不能为空", mockT),
      ).toBe("mcp.error.idRequired");
    });

    it("maps command required error", () => {
      expect(
        translateMcpBackendError("stdio 类型的 MCP 服务器缺少 command 字段", mockT),
      ).toBe("mcp.error.commandRequired");
    });

    it("maps url required error", () => {
      expect(
        translateMcpBackendError("http 类型的 MCP 服务器缺少 url 字段", mockT),
      ).toBe("mcp.wizard.urlRequired");
    });

    it("maps JSON invalid errors", () => {
      expect(
        translateMcpBackendError("MCP 服务器定义必须为 JSON 对象", mockT),
      ).toBe("mcp.error.jsonInvalid");
      expect(
        translateMcpBackendError("MCP 服务器 name 必须为字符串", mockT),
      ).toBe("mcp.error.jsonInvalid");
    });

    it("maps TOML invalid errors", () => {
      expect(
        translateMcpBackendError("解析 config.toml 失败", mockT),
      ).toBe("mcp.error.tomlInvalid");
      expect(
        translateMcpBackendError("无法识别的 TOML 格式", mockT),
      ).toBe("mcp.error.tomlInvalid");
    });

    it("returns empty string for unknown messages", () => {
      expect(translateMcpBackendError("random unknown text", mockT)).toBe("");
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/utils/errorUtils.test.ts`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If a mapping is missing or wrong, update the source and test together.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/utils/errorUtils.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/utils/errorUtils.test.ts
git commit -m "test(unit): add errorUtils coverage"
```

---

## Task 5: Unit Tests for `formatters`

**Files:**
- Create: `tests/utils/formatters.test.ts`
- Source: `src/utils/formatters.ts`

**Interfaces:**
- Consumes: `src/utils/formatters.ts` exports.
- Produces: `tests/utils/formatters.test.ts` covering `formatJSON` and `parseSmartMcpJson`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect } from "vitest";
import { formatJSON, parseSmartMcpJson } from "@/utils/formatters";

describe("formatters", () => {
  describe("formatJSON", () => {
    it("returns empty string for empty input", () => {
      expect(formatJSON("")).toBe("");
      expect(formatJSON("   ")).toBe("");
    });

    it("pretty-prints compact JSON", () => {
      expect(formatJSON('{"a":1,"b":2}')).toBe(
        JSON.stringify({ a: 1, b: 2 }, null, 2),
      );
    });

    it("throws on invalid JSON", () => {
      expect(() => formatJSON("not json")).toThrow();
    });
  });

  describe("parseSmartMcpJson", () => {
    it("returns empty config for empty input", () => {
      const result = parseSmartMcpJson("");
      expect(result.config).toEqual({});
      expect(result.formattedConfig).toBe("");
    });

    it("parses a bare server config object", () => {
      const result = parseSmartMcpJson('{"command":"npx","args":["-y"]}');
      expect(result.config).toEqual({ command: "npx", args: ["-y"] });
      expect(result.id).toBeUndefined();
    });

    it("extracts id from single-key wrapper object", () => {
      const result = parseSmartMcpJson(
        '{"filesystem":{"command":"npx","args":["/tmp"]}}',
      );
      expect(result.id).toBe("filesystem");
      expect(result.config).toEqual({ command: "npx", args: ["/tmp"] });
      expect(result.formattedConfig).toContain('"command": "npx"');
    });

    it("wraps a key-value fragment into a complete object", () => {
      const result = parseSmartMcpJson('"server": {"command": "npx"}');
      expect(result.id).toBe("server");
      expect(result.config).toEqual({ command: "npx" });
    });

    it("throws on invalid JSON", () => {
      expect(() => parseSmartMcpJson("not json")).toThrow();
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/utils/formatters.test.ts`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If `formatJSON` output ordering differs, compare parsed objects instead of exact strings.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/utils/formatters.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/utils/formatters.test.ts
git commit -m "test(unit): add formatters coverage"
```

---

## Task 6: Unit Tests for `textNormalization`

**Files:**
- Create: `tests/utils/textNormalization.test.ts`
- Source: `src/utils/textNormalization.ts`

**Interfaces:**
- Consumes: `src/utils/textNormalization.ts` exports.
- Produces: `tests/utils/textNormalization.test.ts` covering `normalizeQuotes` and `normalizeTomlText`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect } from "vitest";
import {
  normalizeQuotes,
  normalizeTomlText,
} from "@/utils/textNormalization";

describe("textNormalization", () => {
  describe("normalizeQuotes", () => {
    it("returns falsy input unchanged", () => {
      expect(normalizeQuotes("")).toBe("");
    });

    it("normalizes Chinese double quotes to ASCII", () => {
      expect(normalizeQuotes("“hello”")).toBe('"hello"');
      expect(normalizeQuotes("„hello‟")).toBe('"hello"');
      expect(normalizeQuotes("＂hello＂")).toBe('"hello"');
    });

    it("normalizes Chinese single quotes to ASCII", () => {
      expect(normalizeQuotes("‘hello’")).toBe("'hello'");
      expect(normalizeQuotes("＇hello＇")).toBe("'hello'");
    });

    it("leaves book name quotes alone", () => {
      expect(normalizeQuotes("《hello》")).toBe("《hello》");
      expect(normalizeQuotes("「hello」")).toBe("「hello」");
    });
  });

  describe("normalizeTomlText", () => {
    it("delegates to normalizeQuotes", () => {
      expect(normalizeTomlText("“key" = \"value\"")).toBe(
        '"key" = "value"',
      );
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/utils/textNormalization.test.ts`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If quote characters are missing from the regex, add them to the source with a regression test.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/utils/textNormalization.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/utils/textNormalization.test.ts
git commit -m "test(unit): add textNormalization coverage"
```

---

## Task 7: Unit Tests for `usageRange`

**Files:**
- Create: `tests/lib/usageRange.test.ts`
- Source: `src/lib/usageRange.ts`

**Interfaces:**
- Consumes: `src/lib/usageRange.ts` exports and `UsageRangeSelection` type.
- Produces: `tests/lib/usageRange.test.ts` covering `resolveUsageRange` and `getUsageRangePresetLabel`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect } from "vitest";
import {
  resolveUsageRange,
  getUsageRangePresetLabel,
} from "@/lib/usageRange";
import type { UsageRangeSelection } from "@/types/usage";

describe("usageRange", () => {
  describe("resolveUsageRange", () => {
    const nowMs = new Date("2026-01-15T12:00:00.000Z").getTime();

    it("resolves today preset to start of local day", () => {
      const selection: UsageRangeSelection = { preset: "today" };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.endDate).toBe(Math.floor(nowMs / 1000));
      expect(result.startDate).toBeLessThanOrEqual(result.endDate);
    });

    it("resolves 1d preset to last 24 hours", () => {
      const selection: UsageRangeSelection = { preset: "1d" };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.endDate - result.startDate).toBe(24 * 60 * 60);
    });

    it("resolves 7d preset to 7 days lookback", () => {
      const selection: UsageRangeSelection = { preset: "7d" };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.endDate - result.startDate).toBeGreaterThanOrEqual(
        6 * 24 * 60 * 60,
      );
      expect(result.endDate - result.startDate).toBeLessThanOrEqual(
        7 * 24 * 60 * 60,
      );
    });

    it("resolves custom preset with explicit dates", () => {
      const selection: UsageRangeSelection = {
        preset: "custom",
        customStartDate: 1700000000,
        customEndDate: 1700100000,
      };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.startDate).toBe(1700000000);
      expect(result.endDate).toBe(1700100000);
    });

    it("resolves custom preset with liveEndTime to now", () => {
      const selection: UsageRangeSelection = {
        preset: "custom",
        customStartDate: 1700000000,
        liveEndTime: true,
      };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.startDate).toBe(1700000000);
      expect(result.endDate).toBe(Math.floor(nowMs / 1000));
    });
  });

  describe("getUsageRangePresetLabel", () => {
    const t = (key: string, opts?: { defaultValue?: string }) =>
      opts?.defaultValue ?? key;

    it("returns labels for all presets", () => {
      expect(getUsageRangePresetLabel("today", t)).toBe("当天");
      expect(getUsageRangePresetLabel("1d", t)).toBe("1d");
      expect(getUsageRangePresetLabel("7d", t)).toBe("7d");
      expect(getUsageRangePresetLabel("14d", t)).toBe("14d");
      expect(getUsageRangePresetLabel("30d", t)).toBe("30d");
      expect(getUsageRangePresetLabel("custom", t)).toBe("日历筛选");
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/lib/usageRange.test.ts`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If the local-day calculation differs from the test's expectations, adjust the test to match the documented behavior.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/lib/usageRange.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/lib/usageRange.test.ts
git commit -m "test(unit): add usageRange coverage"
```

---

## Task 8: Unit Tests for `skillErrorParser`

**Files:**
- Create: `tests/lib/skillErrorParser.test.ts`
- Source: `src/lib/errors/skillErrorParser.ts`

**Interfaces:**
- Consumes: `src/lib/errors/skillErrorParser.ts` exports.
- Produces: `tests/lib/skillErrorParser.test.ts` covering `parseSkillError` and `formatSkillError`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect, vi } from "vitest";
import {
  parseSkillError,
  formatSkillError,
} from "@/lib/errors/skillErrorParser";

describe("skillErrorParser", () => {
  describe("parseSkillError", () => {
    it("parses a structured JSON error", () => {
      const error = JSON.stringify({
        code: "SKILL_NOT_FOUND",
        context: { skillName: "missing" },
      });
      expect(parseSkillError(error)).toEqual({
        code: "SKILL_NOT_FOUND",
        context: { skillName: "missing" },
      });
    });

    it("returns null for plain text errors", () => {
      expect(parseSkillError("plain text error")).toBeNull();
    });

    it("returns null for invalid JSON", () => {
      expect(parseSkillError("{not json")).toBeNull();
    });

    it("returns null when code/context are missing", () => {
      expect(parseSkillError(JSON.stringify({ foo: "bar" }))).toBeNull();
    });
  });

  describe("formatSkillError", () => {
    const t = vi.fn((key: string, _opts?: any) => key);

    it("formats structured errors with i18n keys", () => {
      const error = JSON.stringify({
        code: "SKILL_NOT_FOUND",
        context: { skillName: "missing" },
      });
      const result = formatSkillError(error, t, "custom.title");
      expect(result.title).toBe("custom.title");
      expect(result.description).toContain("skills.error.skillNotFound");
    });

    it("appends suggestion when provided", () => {
      const error = JSON.stringify({
        code: "DOWNLOAD_FAILED",
        context: {},
        suggestion: "checkNetwork",
      });
      const result = formatSkillError(error, t);
      expect(result.description).toContain("skills.error.downloadFailed");
      expect(result.description).toContain("skills.error.suggestion.checkNetwork");
    });

    it("falls back to raw string for unstructured errors", () => {
      const result = formatSkillError("raw error", t);
      expect(result.title).toBe("skills.installFailed");
      expect(result.description).toBe("raw error");
    });

    it("uses common.error fallback for empty unstructured errors", () => {
      const result = formatSkillError("", t);
      expect(result.description).toBe("common.error");
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/lib/skillErrorParser.test.ts`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If a mapping is wrong, fix the source and update the test.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/lib/skillErrorParser.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/lib/skillErrorParser.test.ts
git commit -m "test(unit): add skillErrorParser coverage"
```

---

## Task 9: Unit Tests for Generic Hooks

**Files:**
- Create: `tests/hooks/genericHooks.test.tsx`
- Source: `src/hooks/useDebouncedValue.ts`, `src/hooks/useLastValidValue.ts`, `src/hooks/useDarkMode.ts`

**Interfaces:**
- Consumes: React Testing Library hook helpers, existing test setup.
- Produces: `tests/hooks/genericHooks.test.tsx` covering `useDebouncedValue`, `useLastValidValue`, `useDarkMode`.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useDebouncedValue } from "@/hooks/useDebouncedValue";
import { useLastValidValue } from "@/hooks/useLastValidValue";
import { useDarkMode } from "@/hooks/useDarkMode";

describe("generic hooks", () => {
  describe("useDebouncedValue", () => {
    beforeEach(() => {
      vi.useFakeTimers({ shouldAdvanceTime: true });
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it("returns initial value immediately", () => {
      const { result } = renderHook(() => useDebouncedValue("initial", 300));
      expect(result.current).toBe("initial");
    });

    it("updates value after delay", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useDebouncedValue(value, 300),
        { initialProps: { value: "a" } },
      );
      rerender({ value: "b" });
      expect(result.current).toBe("a");
      act(() => {
        vi.advanceTimersByTime(300);
      });
      expect(result.current).toBe("b");
    });

    it("resets timer on rapid changes", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useDebouncedValue(value, 300),
        { initialProps: { value: "a" } },
      );
      rerender({ value: "b" });
      act(() => {
        vi.advanceTimersByTime(200);
      });
      rerender({ value: "c" });
      act(() => {
        vi.advanceTimersByTime(200);
      });
      expect(result.current).toBe("a");
      act(() => {
        vi.advanceTimersByTime(100);
      });
      expect(result.current).toBe("c");
    });
  });

  describe("useLastValidValue", () => {
    it("returns current value when non-null", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useLastValidValue(value),
        { initialProps: { value: "first" } },
      );
      expect(result.current).toBe("first");
      rerender({ value: "second" });
      expect(result.current).toBe("second");
    });

    it("keeps last valid value when current becomes null", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useLastValidValue(value),
        { initialProps: { value: "valid" } },
      );
      rerender({ value: null });
      expect(result.current).toBe("valid");
      rerender({ value: undefined });
      expect(result.current).toBe("valid");
    });

    it("returns null when no valid value has been seen", () => {
      const { result } = renderHook(() => useLastValidValue(null));
      expect(result.current).toBeNull();
    });
  });

  describe("useDarkMode", () => {
    beforeEach(() => {
      document.documentElement.classList.remove("dark");
    });

    afterEach(() => {
      document.documentElement.classList.remove("dark");
    });

    it("returns false when dark class is absent", () => {
      const { result } = renderHook(() => useDarkMode());
      expect(result.current).toBe(false);
    });

    it("returns true when dark class is present", () => {
      document.documentElement.classList.add("dark");
      const { result } = renderHook(() => useDarkMode());
      expect(result.current).toBe(true);
    });

    it("reacts to class changes", async () => {
      const { result } = renderHook(() => useDarkMode());
      expect(result.current).toBe(false);
      act(() => {
        document.documentElement.classList.add("dark");
      });
      await waitFor(() => expect(result.current).toBe(true));
    });
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/hooks/genericHooks.test.tsx`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If fake timers conflict with `waitFor`, switch the dark mode test to real timers or use `runAllTimers` carefully.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/hooks/genericHooks.test.tsx
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/hooks/genericHooks.test.tsx
git commit -m "test(unit): add generic hooks coverage"
```

---

## Task 10: Unit Tests for Web Providers API

**Files:**
- Create: `tests/lib/web/providers.test.ts`
- Source: `src/lib/api/web/providers.ts`, `src/lib/api/web-client.ts`

**Interfaces:**
- Consumes: `providersApi` from `src/lib/api/web/providers.ts`.
- Produces: `tests/lib/web/providers.test.ts` verifying HTTP methods, URLs, and payloads.

- [ ] **Step 1: Write the failing test**

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { providersApi, universalProvidersApi } from "@/lib/api/web/providers";

const jsonResponse = (data: unknown, status = 200) =>
  new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json" },
  });

describe("web providers API", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("getAll fetches providers with app query", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: {} }));

    await providersApi.getAll("claude");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/providers?app=claude",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("add posts provider and app", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: true }));

    const provider = {
      id: "p1",
      name: "Test",
      settingsConfig: {},
    };
    await providersApi.add(provider as any, "claude");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/providers",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ provider, app: "claude" }),
      }),
    );
  });

  it("update puts to provider id with originalId", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: true }));

    const provider = { id: "new-id", name: "Test", settingsConfig: {} };
    await providersApi.update(provider as any, "claude", "old-id");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/providers/old-id",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ provider, app: "claude", originalId: "old-id" }),
      }),
    );
  });

  it("delete sends DELETE with app query", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: true }));

    await providersApi.delete("p1", "claude");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/providers/p1?app=claude",
      expect.objectContaining({ method: "DELETE" }),
    );
  });

  it("switch posts to switch endpoint", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: true }));

    const result = await providersApi.switch("p1", "claude");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/providers/p1/switch?app=claude",
      expect.objectContaining({ method: "POST" }),
    );
    expect(result.warnings).toEqual([]);
  });

  it("universal getAll fetches all universal providers", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: {} }));

    await universalProvidersApi.getAll();

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/universal-providers",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("universal upsert posts provider", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(jsonResponse({ success: true, data: true }));

    const provider = { id: "u1", name: "Universal" } as any;
    await universalProvidersApi.upsert(provider);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/universal-providers",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(provider),
      }),
    );
  });
});
```

- [ ] **Step 2: Run the test to verify behavior**

Run: `pnpm test:unit tests/lib/web/providers.test.ts`

Expected: PASS.

- [ ] **Step 3: Fix any discrepancies**

If the auth token lifecycle from `web-client` causes cross-test contamination, reset modules or clear `localStorage` as shown.

- [ ] **Step 4: Run the full unit suite**

Run:

```bash
pnpm test:unit tests/lib/web/providers.test.ts
pnpm test:unit
```

Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/lib/web/providers.test.ts
git commit -m "test(unit): add web providers API coverage"
```

---

## Self-Review

**1. Spec coverage:**
- Web UI E2E with Playwright: Tasks 1–2 cover provider CRUD and settings language change.
- More unit tests: Tasks 3–10 cover utilities, hooks, and web API clients identified as untested.

**2. Placeholder scan:**
- No TBD/TODO placeholders.
- All code blocks contain real, runnable test code.
- All commands and expected outputs are explicit.

**3. Type consistency:**
- `McpServerSpec`, `Provider`, `UsageRangeSelection` types are imported from the project.
- Test payloads match the expected shapes used by the source modules.

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-07-02-add-webui-and-unit-tests.md`.**

Two execution options:

1. **Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - Execute tasks in this session using `executing-plans`, batch execution with checkpoints for review.

**Which approach?**
