import { get, post, del } from "../web-client";
import type { McpServer } from "@/types";
import type { AppId } from "./types";

export const mcpApi = {
  async getServers(): Promise<McpServer[]> {
    return get("/mcp");
  },

  async getServer(id: string): Promise<McpServer | null> {
    return get(`/mcp/${id}`);
  },

  async upsertServer(server: McpServer): Promise<boolean> {
    return post("/mcp", server);
  },

  async deleteServer(id: string): Promise<boolean> {
    return del(`/mcp/${id}`);
  },

  async toggleServer(id: string, enabled: boolean): Promise<boolean> {
    return post(`/mcp/${id}/toggle`, { enabled });
  },

  async importFromApps(appId: AppId): Promise<number> {
    return post("/mcp/import", { app: appId });
  },

  async getClaudeMcpStatus(): Promise<{ enabled: boolean } | null> {
    return get("/mcp/claude/status");
  },

  async getCodexMcpStatus(): Promise<{ enabled: boolean } | null> {
    return get("/mcp/codex/status");
  },

  async getGeminiMcpStatus(): Promise<{ enabled: boolean } | null> {
    return get("/mcp/gemini/status");
  },

  async validateCommand(
    command: string,
  ): Promise<{ valid: boolean; error?: string }> {
    return post("/mcp/validate", { command });
  },
};
