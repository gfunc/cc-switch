import { get, post, put, del, connectWebSocket } from "../web-client";
import type { Settings, WebDavSyncSettings, RemoteSnapshotInfo } from "@/types";
import type { AppId } from "./types";

export interface ConfigTransferResult {
  success: boolean;
  message: string;
  filePath?: string;
  backupId?: string;
}

export interface WebDavTestResult {
  success: boolean;
  message?: string;
}

export interface WebDavSyncResult {
  status: string;
}

export interface RectifierConfig {
  enabled: boolean;
  requestThinkingSignature: boolean;
  requestThinkingBudget: boolean;
}

export interface LogConfig {
  enabled: boolean;
  level: "error" | "warn" | "info" | "debug" | "trace";
}

export const settingsApi = {
  async get(): Promise<Settings> {
    return get("/settings");
  },

  async save(settings: Settings): Promise<boolean> {
    return put("/settings", settings);
  },

  async restart(): Promise<boolean> {
    console.warn("restart_app not available in web mode");
    return true;
  },

  async checkUpdates(): Promise<void> {
    console.warn("check_for_updates not available in web mode");
  },

  async isPortable(): Promise<boolean> {
    return false;
  },

  async getConfigDir(appId: AppId): Promise<string> {
    try {
      const response = await get<{ path: string }>(
        `/settings/config-dir?app=${appId}`,
      );
      return typeof response?.path === "string" ? response.path : "";
    } catch (error) {
      console.warn(
        "getConfigDir endpoint unavailable in web mode, using default path fallback",
        error,
      );
      return "";
    }
  },

  async openConfigFolder(_appId: AppId): Promise<void> {
    console.warn("open_config_folder not available in web mode");
  },

  async selectConfigDirectory(defaultPath?: string): Promise<string | null> {
    console.warn("pick_directory not available in web mode");
    return defaultPath || null;
  },

  async getClaudeCodeConfigPath(): Promise<string> {
    const response = await get<{ path: string }>("/settings/claude-code-path");
    return response.path;
  },

  async getAppConfigPath(): Promise<string> {
    const response = await get<{ path: string }>("/settings/app-config-path");
    return response.path;
  },

  async openAppConfigFolder(): Promise<void> {
    console.warn("open_app_config_folder not available in web mode");
  },

  async getAppConfigDirOverride(): Promise<string | null> {
    try {
      const response = await get<{ path: string | null }>(
        "/settings/app-config-dir-override",
      );
      return response?.path ?? null;
    } catch (error) {
      console.warn(
        "getAppConfigDirOverride endpoint unavailable in web mode, using null override",
        error,
      );
      return null;
    }
  },

  async setAppConfigDirOverride(path: string | null): Promise<boolean> {
    return post("/settings/app-config-dir-override", { path });
  },

  async applyClaudePluginConfig(options: {
    official: boolean;
  }): Promise<boolean> {
    return post("/settings/apply-claude-plugin", options);
  },

  async applyClaudeOnboardingSkip(): Promise<boolean> {
    return post("/settings/claude-onboarding-skip", {});
  },

  async clearClaudeOnboardingSkip(): Promise<boolean> {
    return del("/settings/claude-onboarding-skip");
  },

  async saveFileDialog(defaultName: string): Promise<string | null> {
    console.warn("save_file_dialog not available in web mode");
    return defaultName;
  },

  async openFileDialog(): Promise<string | null> {
    console.warn("open_file_dialog not available in web mode");
    return null;
  },

  async exportConfigToFile(filePath: string): Promise<ConfigTransferResult> {
    return post("/settings/export", { filePath });
  },

  async importConfigFromFile(filePath: string): Promise<ConfigTransferResult> {
    return post("/settings/import", { filePath });
  },

  async webdavTestConnection(
    settings: WebDavSyncSettings,
    preserveEmptyPassword = true,
  ): Promise<WebDavTestResult> {
    return post("/settings/webdav/test", { settings, preserveEmptyPassword });
  },

  async webdavSyncUpload(): Promise<WebDavSyncResult> {
    return post("/settings/webdav/upload", {});
  },

  async webdavSyncDownload(): Promise<WebDavSyncResult> {
    return post("/settings/webdav/download", {});
  },

  async webdavSyncSaveSettings(
    settings: WebDavSyncSettings,
    passwordTouched = false,
  ): Promise<{ success: boolean }> {
    return post("/settings/webdav/settings", { settings, passwordTouched });
  },

  async webdavSyncFetchRemoteInfo(): Promise<
    RemoteSnapshotInfo | { empty: true }
  > {
    return get("/settings/webdav/remote-info");
  },

  async syncCurrentProvidersLive(): Promise<void> {
    await post("/settings/sync-providers-live", {});
  },

  async openExternal(url: string): Promise<void> {
    window.open(url, "_blank");
  },

  async setAutoLaunch(enabled: boolean): Promise<boolean> {
    console.warn("set_auto_launch not available in web mode");
    return enabled;
  },

  async getAutoLaunchStatus(): Promise<boolean> {
    return false;
  },

  async getToolVersions(): Promise<
    Array<{
      name: string;
      version: string | null;
      latest_version: string | null;
      error: string | null;
    }>
  > {
    return get("/settings/tool-versions");
  },

  async getRectifierConfig(): Promise<RectifierConfig> {
    return get("/settings/rectifier-config");
  },

  async setRectifierConfig(config: RectifierConfig): Promise<boolean> {
    return put("/settings/rectifier-config", config);
  },

  async getLogConfig(): Promise<LogConfig> {
    return get("/settings/log-config");
  },

  async setLogConfig(config: LogConfig): Promise<boolean> {
    return put("/settings/log-config", config);
  },

  onWebDavSyncStatusUpdated(
    callback: (status: { status: string; error?: string }) => void,
  ): () => void {
    return connectWebSocket((data: any) => {
      if (data.event === "webdav-sync-status-updated") {
        callback(data.data);
      }
    });
  },
};
