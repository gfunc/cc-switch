import { isTauri } from "@/lib/environment";

import {
  providersApi as tauriProvidersApi,
  universalProvidersApi as tauriUniversalProvidersApi,
} from "./providers";
import { settingsApi as tauriSettingsApi } from "./settings";
import { mcpApi as tauriMcpApi } from "./mcp";
import { promptsApi } from "./prompts";
import { skillsApi } from "./skills";
import { usageApi } from "./usage";
import { vscodeApi } from "./vscode";
import { proxyApi } from "./proxy";
import { openclawApi } from "./openclaw";
import { sessionsApi } from "./sessions";
import { workspaceApi } from "./workspace";
import * as configApi from "./config";

import {
  providersApi as webProvidersApi,
  universalProvidersApi as webUniversalProvidersApi,
} from "./web/providers";
import { settingsApi as webSettingsApi } from "./web/settings";
import { mcpApi as webMcpApi } from "./web/mcp";

export type { AppId } from "./types";
export type { ProviderSwitchEvent } from "./providers";
export type { Prompt } from "./prompts";

// Runtime API selection based on environment
// Desktop app uses Tauri APIs, Web uses HTTP APIs
export const providersApi = isTauri() ? tauriProvidersApi : webProvidersApi;
export const universalProvidersApi = isTauri()
  ? tauriUniversalProvidersApi
  : webUniversalProvidersApi;
export const settingsApi = isTauri() ? tauriSettingsApi : webSettingsApi;
export const mcpApi = isTauri() ? tauriMcpApi : webMcpApi;
export { promptsApi };
export { skillsApi };
export { usageApi };
export { vscodeApi };
export { proxyApi };
export { openclawApi };
export { sessionsApi };
export { workspaceApi };
export { configApi };
