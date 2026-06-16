import { get, post } from "../web-client";

export const webAuthApi = {
  async generateToken(): Promise<string> {
    return post("/auth/generate", {});
  },
  async logout(): Promise<void> {
    return post("/auth/logout", {});
  },
  async isTokenRevealEnabled(): Promise<boolean> {
    return get("/auth/token-reveal-enabled");
  },
};
