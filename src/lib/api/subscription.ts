import { invoke } from "@tauri-apps/api/core";
import type { SubscriptionQuota } from "@/types/subscription";
import { isTauri } from "@/lib/environment";
import { post } from "@/lib/api/web-client";

export const subscriptionApi = {
  getQuota: (tool: string): Promise<SubscriptionQuota> => {
    if (!isTauri()) {
      // Official subscription quota relies on desktop-managed CLI/OAuth
      // credentials that don't exist in the web server.
      return Promise.reject(
        new Error("Official subscription quota is not available in web mode"),
      );
    }
    return invoke("get_subscription_quota", { tool });
  },
  getCodexOauthQuota: (
    accountId: string | null,
  ): Promise<SubscriptionQuota> => {
    if (!isTauri()) {
      return Promise.reject(
        new Error("Codex OAuth quota is not available in web mode"),
      );
    }
    return invoke("get_codex_oauth_quota", { accountId });
  },
  getXaiOauthQuota: (accountId: string | null): Promise<SubscriptionQuota> => {
    if (!isTauri()) {
      return Promise.reject(
        new Error("xAI OAuth quota is not available in web mode"),
      );
    }
    return invoke("get_xai_oauth_quota", { accountId });
  },
  getCodingPlanQuota: (
    baseUrl: string,
    apiKey: string,
    accessKeyId?: string,
    secretAccessKey?: string,
    // 智谱团队版（zhipu_team）靠显式标识路由（base_url 与个人版相同无法区分）。
    codingPlanProvider?: string,
    teamOrganizationId?: string,
    teamProjectId?: string,
  ): Promise<SubscriptionQuota> => {
    if (!isTauri()) {
      return post<SubscriptionQuota>("/providers/usage/coding-plan", {
        baseUrl,
        apiKey,
        accessKeyId,
        secretAccessKey,
        codingPlanProvider,
      });
    }
    return invoke("get_coding_plan_quota", {
      baseUrl,
      apiKey,
      accessKeyId,
      secretAccessKey,
      codingPlanProvider,
      teamOrganizationId,
      teamProjectId,
    });
  },
  getBalance: (
    baseUrl: string,
    apiKey: string,
  ): Promise<import("@/types").UsageResult> => {
    if (!isTauri()) {
      return post<import("@/types").UsageResult>("/providers/usage/balance", {
        baseUrl,
        apiKey,
      });
    }
    return invoke("get_balance", { baseUrl, apiKey });
  },
};
