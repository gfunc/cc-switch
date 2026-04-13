import { post } from "../web-client";

export const webAuthApi = {
  async generateToken(): Promise<string> {
    return post("/auth/generate", {});
  },
};
