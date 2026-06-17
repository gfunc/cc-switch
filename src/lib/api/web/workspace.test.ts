import { describe, it, expect, vi, beforeEach } from "vitest";
import { workspaceApi } from "./workspace";

const mocks = vi.hoisted(() => ({
  get: vi.fn(),
  put: vi.fn(),
  del: vi.fn(),
}));

vi.mock("../web-client", () => ({
  get: mocks.get,
  put: mocks.put,
  del: mocks.del,
}));

describe("web workspaceApi", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("reads a workspace file", async () => {
    mocks.get.mockResolvedValueOnce("# agents");
    const result = await workspaceApi.readFile("AGENTS.md");
    expect(result).toBe("# agents");
    expect(mocks.get).toHaveBeenCalledWith("/workspace/file/AGENTS.md");
  });

  it("lists daily memory files", async () => {
    mocks.get.mockResolvedValueOnce([{ filename: "2026-06-18.md" }]);
    const result = await workspaceApi.listDailyMemoryFiles();
    expect(result).toHaveLength(1);
    expect(mocks.get).toHaveBeenCalledWith("/workspace/daily-memory");
  });
});
