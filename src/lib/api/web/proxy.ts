import { get, post } from "../web-client";
import type {
  ProxyConfig,
  ProxyStatus,
  ProxyServerInfo,
  ProxyTakeoverStatus,
  GlobalProxyConfig,
  AppProxyConfig,
} from "@/types/proxy";

export const proxyApi = {
  async startProxyServer(): Promise<ProxyServerInfo> {
    return post("/proxy/start", {});
  },

  async stopProxyWithRestore(): Promise<void> {
    await post("/proxy/stop", {});
  },

  async getProxyStatus(): Promise<ProxyStatus> {
    return get("/proxy/status");
  },

  async isProxyRunning(): Promise<boolean> {
    const status = await get<ProxyStatus>("/proxy/status");
    return status.running;
  },

  async isLiveTakeoverActive(): Promise<boolean> {
    const status = await get<ProxyTakeoverStatus>("/proxy/takeover");
    return Object.values(status).some((v) => v === true);
  },

  async switchProxyProvider(
    _appType: string,
    _providerId: string,
  ): Promise<void> {
    console.warn("switch_proxy_provider not available in web mode");
  },

  async getProxyTakeoverStatus(): Promise<ProxyTakeoverStatus> {
    return get("/proxy/takeover");
  },

  async setProxyTakeoverForApp(
    appType: string,
    enabled: boolean,
  ): Promise<void> {
    await post("/proxy/takeover", { app: appType, enabled });
  },

  async getProxyConfig(): Promise<ProxyConfig> {
    return get("/proxy/config");
  },

  async updateProxyConfig(config: ProxyConfig): Promise<void> {
    await post("/proxy/config", config);
  },

  async getGlobalProxyConfig(): Promise<GlobalProxyConfig> {
    return get("/proxy/config/global");
  },

  async updateGlobalProxyConfig(config: GlobalProxyConfig): Promise<void> {
    await post("/proxy/config/global", config);
  },

  async getProxyConfigForApp(appType: string): Promise<AppProxyConfig> {
    return get(`/proxy/config/app?app=${appType}`);
  },

  async updateProxyConfigForApp(config: AppProxyConfig): Promise<void> {
    await post(`/proxy/config/app?app=${config.appType}`, config);
  },

  async getDefaultCostMultiplier(_appType: string): Promise<string> {
    return "1.0";
  },

  async setDefaultCostMultiplier(
    _appType: string,
    _value: string,
  ): Promise<void> {
    console.warn("set_default_cost_multiplier not available in web mode");
  },

  async getPricingModelSource(_appType: string): Promise<string> {
    return "default";
  },

  async setPricingModelSource(_appType: string, _value: string): Promise<void> {
    console.warn("set_pricing_model_source not available in web mode");
  },
};
