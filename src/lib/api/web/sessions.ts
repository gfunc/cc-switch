import { get } from "../web-client";
import type { SessionMeta, SessionMessage } from "@/types";

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

  async launchTerminal(_options: {
    command: string;
    cwd?: string | null;
    customConfig?: string | null;
  }): Promise<boolean> {
    console.warn("launch_session_terminal not available in web mode");
    return false;
  },
};
