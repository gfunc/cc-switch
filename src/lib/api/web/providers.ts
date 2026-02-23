import { get, post, put, del, connectWebSocket, getAuthToken } from "../web-client";
import type {
  Provider,
  UniversalProvider,
  UniversalProvidersMap,
} from "@/types";
import type { AppId } from "./types";
import type { ClaudeDesktopStatus, SwitchResult } from "../providers";
import type { ClaudeDesktopDefaultRoute } from "../providers";

export interface ProviderSortUpdate {
  id: string;
  sortIndex: number;
}

export interface ProviderSwitchEvent {
  appType: AppId;
  providerId: string;
}

export const providersApi = {
  async getAll(appId: AppId): Promise<Record<string, Provider>> {
    return get(`/providers?app=${appId}`);
  },

  async getCurrent(appId: AppId): Promise<string | null> {
    return get(`/providers/current?app=${appId}`);
  },

  async add(
    provider: Provider,
    appId: AppId,
    addToLive?: boolean,
  ): Promise<boolean> {
    return post("/providers", { provider, app: appId, addToLive });
  },

  async update(
    provider: Provider,
    appId: AppId,
    originalId?: string,
  ): Promise<boolean> {
    return put(`/providers/${originalId ?? provider.id}`, {
      provider,
      app: appId,
      originalId,
    });
  },

  async delete(id: string, appId: AppId): Promise<boolean> {
    return del(`/providers/${id}?app=${appId}`);
  },

  async removeFromLiveConfig(id: string, appId: AppId): Promise<boolean> {
    return post(`/providers/${id}/remove-from-live`, { app: appId });
  },

  async switch(id: string, appId: AppId): Promise<SwitchResult> {
    await post(`/providers/${id}/switch?app=${encodeURIComponent(appId)}`, {});
    return { warnings: [] };
  },

  async importDefault(appId: AppId): Promise<boolean> {
    return post("/providers/import-default", { app: appId });
  },

  async updateTrayMenu(): Promise<boolean> {
    // Tray menu is a desktop-only concept; no-op in web mode.
    return true;
  },

  async updateSortOrder(
    updates: ProviderSortUpdate[],
    appId: AppId,
  ): Promise<boolean> {
    return post("/providers/sort", { updates, app: appId });
  },

  onSwitched(handler: (event: ProviderSwitchEvent) => void): () => void {
    return connectWebSocket((data: any) => {
      if (data.event === "provider.switched") {
        handler({
          appType: data.data.app,
          providerId: data.data.id,
        });
      }
    });
  },

  async openTerminal(
    _providerId: string,
    _appId: AppId,
    _options?: { cwd?: string },
  ): Promise<boolean> {
    console.warn("open_provider_terminal not available in web mode");
    return false;
  },

  async importOpenCodeFromLive(): Promise<number> {
    return post("/providers/import-opencode-live", {});
  },

  async getOpenCodeLiveProviderIds(): Promise<string[]> {
    return get("/providers/opencode-live-ids");
  },

  async getOpenClawLiveProviderIds(): Promise<string[]> {
    return get("/providers/openclaw-live-ids");
  },

<<<<<<< HEAD
  async getHermesLiveProviderIds(): Promise<string[]> {
    return get("/providers/hermes-live-ids");
  },

  async getClaudeDesktopStatus(): Promise<ClaudeDesktopStatus> {
    return get("/providers/claude-desktop-status");
  },

  async getClaudeDesktopDefaultRoutes(): Promise<ClaudeDesktopDefaultRoute[]> {
    return get("/providers/claude-desktop-default-routes");
  },

  async importOpenClawFromLive(): Promise<number> {
    return post("/providers/import-openclaw-live", {});
  },

  async importHermesFromLive(): Promise<number> {
    return post("/providers/import-hermes-live", {});
  },

  async importKimiFromLive(): Promise<number> {
    return post("/providers/import-kimi-live", {});
  },

  async getKimiLiveProviderIds(): Promise<string[]> {
    return get("/providers/kimi-live-ids");
  },

  async importClaudeDesktopFromClaude(): Promise<number> {
    return post("/providers/import-claude-desktop-from-claude", {});
  },

  async ensureClaudeDesktopOfficialProvider(): Promise<boolean> {
    return post("/providers/ensure-claude-desktop-official", {});
  },

  async ensureCodexOfficialProvider(): Promise<boolean> {
    return post("/providers/ensure-codex-official", {});
  },

  async ensureGrokBuildOfficialProvider(): Promise<boolean> {
    return post("/providers/ensure-grokbuild-official", {});
  },

  async importFromUpload(appId: AppId, file: File): Promise<boolean> {
    const formData = new FormData();
    formData.append("config", file);

    const base = import.meta.env.VITE_API_BASE_URL || "/api/v1";
    const response = await fetch(`${base}/providers/import-upload?app=${appId}`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${getAuthToken() || ""}`,
      },
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to import provider");
    }

    const result = await response.json();
    if (!result.success) {
      throw new Error(result.error || "Failed to import provider");
    }
    return result.data;
  },
};

export const universalProvidersApi = {
  async getAll(): Promise<UniversalProvidersMap> {
    return get("/universal-providers");
  },

  async get(id: string): Promise<UniversalProvider | null> {
    return get(`/universal-providers/${id}`);
  },

  async upsert(provider: UniversalProvider): Promise<boolean> {
    return post("/universal-providers", provider);
  },

  async delete(id: string): Promise<boolean> {
    return del(`/universal-providers/${id}`);
  },

  async sync(id: string): Promise<boolean> {
    return post(`/universal-providers/${id}/sync`, {});
  },
};
