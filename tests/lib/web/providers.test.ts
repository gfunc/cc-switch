import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { providersApi, universalProvidersApi } from "@/lib/api/web/providers";
import type { Provider, UniversalProvider } from "@/types";

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

    const provider: Provider = {
      id: "p1",
      name: "Test",
      settingsConfig: {},
    };
    await providersApi.add(provider, "claude");

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

    const provider: Provider = {
      id: "new-id",
      name: "Test",
      settingsConfig: {},
    };
    await providersApi.update(provider, "claude", "old-id");

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

    const provider: UniversalProvider = {
      id: "u1",
      name: "Universal",
      providerType: "custom",
      apps: { claude: true, codex: false, gemini: false },
      baseUrl: "http://localhost:3000",
      apiKey: "test-key",
      models: {},
    };
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
