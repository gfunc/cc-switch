const isWebMode = import.meta.env.VITE_CC_SWITCH_MODE === "web";

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

export const providersApi = isWebMode ? webProvidersApi : tauriProvidersApi;
export const universalProvidersApi = isWebMode
  ? webUniversalProvidersApi
  : tauriUniversalProvidersApi;
export const settingsApi = isWebMode ? webSettingsApi : tauriSettingsApi;
export const mcpApi = isWebMode ? webMcpApi : tauriMcpApi;
export { promptsApi };
export { skillsApi };
export { usageApi };
export { vscodeApi };
export { proxyApi };
export { openclawApi };
export { sessionsApi };
export { workspaceApi };
export { configApi };
