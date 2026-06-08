import { get, del } from "../web-client";
import type { SessionMeta, SessionMessage } from "@/types";
import type { DeleteSessionOptions, DeleteSessionResult } from "../sessions";

export const sessionsApi = {
  async list(): Promise<SessionMeta[]> {
    return get("/sessions");
  },

  async getMessages(
    providerId: string,
    sourcePath: string,
  ): Promise<SessionMessage[]> {
    return get(
      `/sessions/${providerId}/messages?sourcePath=${encodeURIComponent(sourcePath)}`,
    );
  },

  async delete(options: DeleteSessionOptions): Promise<boolean> {
    await del(`/sessions/${encodeURIComponent(options.sessionId)}`);
    return true;
  },

  async deleteMany(
    items: DeleteSessionOptions[],
  ): Promise<DeleteSessionResult[]> {
    return Promise.all(
      items.map(async (item) => {
        try {
          await this.delete(item);
          return { ...item, success: true };
        } catch (error) {
          return {
            ...item,
            success: false,
            error: error instanceof Error ? error.message : String(error),
          };
        }
      }),
    );
  },

  async launchTerminal(_options: {
    command: string;
    cwd?: string | null;
    customConfig?: string | null;
  }): Promise<boolean> {
    console.warn("launch_session_terminal not available in web mode");
    return false;
  },
};
