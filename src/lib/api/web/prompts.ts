import { get, post, del } from "../web-client";
import type { Prompt } from "../prompts";
import type { AppId } from "../types";

export const promptsApi = {
  async getPrompts(appId: AppId): Promise<Prompt[]> {
    return get(`/prompts?app=${appId}`);
  },

  async getPrompt(id: string): Promise<Prompt | null> {
    return get(`/prompts/${id}`);
  },

  async upsertPrompt(prompt: Prompt, appId: AppId): Promise<boolean> {
    return post("/prompts", { prompt, app: appId });
  },

  async deletePrompt(id: string, appId: AppId): Promise<boolean> {
    return del(`/prompts/${id}?app=${appId}`);
  },

  async enablePrompt(id: string, appId: AppId): Promise<boolean> {
    return post(`/prompts/${id}/activate`, { app: appId });
  },

  async importFromFile(filePath: string, appId: AppId): Promise<Prompt> {
    return post("/prompts/import", { filePath, app: appId });
  },

  async getCurrentPromptFileContent(appId: AppId): Promise<string> {
    const response = await get<{ content: string }>(
      `/prompts/current-content?app=${appId}`,
    );
    return response.content;
  },
};
