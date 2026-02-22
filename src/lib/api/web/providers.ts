import { get, post, put, del, connectWebSocket } from "../web-client";
import type { Provider } from "@/types";
export type AppId = "claude" | "codex" | "gemini" | "opencode" | "openclaw";

export interface ProviderSwitchEvent {
  appType: AppId;
  providerId: string;
  timestamp: number;
}

export async function listProviders(
  app: AppId,
): Promise<Record<string, Provider>> {
  return get(`/providers?app=${app}`);
}

export async function getProvider(id: string): Promise<Provider | null> {
  return get(`/providers/${id}`);
}

export async function createProvider(
  app: AppId,
  provider: Omit<Provider, "id" | "createdAt">,
): Promise<string> {
  return post("/providers", { ...provider, app });
}

export async function updateProvider(
  id: string,
  provider: Partial<Provider>,
): Promise<boolean> {
  return put(`/providers/${id}`, provider);
}

export async function deleteProvider(id: string): Promise<boolean> {
  return del(`/providers/${id}`);
}

export async function switchProvider(app: AppId, id: string): Promise<boolean> {
  return post(`/providers/${id}/switch?app=${app}`);
}

export async function getCurrentProvider(app: AppId): Promise<string | null> {
  return get(`/providers/current?app=${app}`);
}

export function onProviderSwitched(
  callback: (event: ProviderSwitchEvent) => void,
): () => void {
  return connectWebSocket((data: any) => {
    if (data.event === "provider.switched") {
      callback({
        appType: data.data.app,
        providerId: data.data.id,
        timestamp: data.timestamp,
      });
    }
  });
}
