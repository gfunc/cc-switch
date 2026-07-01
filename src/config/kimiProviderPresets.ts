/**
 * Kimi Code CLI provider presets configuration
 *
 * Kimi uses additive provider management in `~/.kimi-code/config.toml`:
 * every provider is kept under `[providers."<name>"]` and its models under
 * `[models."<name>/<model-id>"]`. Switching providers only updates
 * `default_model`.
 */
import type { ProviderCategory } from "../types";
import type { PresetTheme, TemplateValueConfig } from "./claudeProviderPresets";

/**
 * A model entry for a Kimi provider.
 *
 * Serialized to TOML as a separate `[models."<provider>/<id>"]` table.
 */
export interface KimiModel {
  /** Model ID — becomes the suffix of the TOML model alias. */
  id: string;
  /** Optional display label (written as `display_name` in TOML). */
  name?: string;
  /** Override the auto-detected context window. */
  max_context_size?: number;
  /** Optional capability flags (e.g. "thinking", "image_in"). */
  capabilities?: string[];
}

export interface KimiProviderSettingsConfig {
  /** Provider key used in `[providers."<provider_name>"]` and model aliases. */
  provider_name: string;
  /** Provider type — usually "kimi". */
  type?: string;
  base_url?: string;
  api_key?: string;
  /** UI-side ordered list; serialized to per-model TOML tables. */
  models?: KimiModel[];
  [key: string]: unknown;
}

export interface KimiProviderPreset {
  name: string;
  nameKey?: string;
  websiteUrl: string;
  apiKeyUrl?: string;
  settingsConfig: KimiProviderSettingsConfig;
  isOfficial?: boolean;
  isPartner?: boolean;
  primePartner?: boolean;
  partnerPromotionKey?: string;
  category?: ProviderCategory;
  templateValues?: Record<string, TemplateValueConfig>;
  theme?: PresetTheme;
  icon?: string;
  iconColor?: string;
  endpointCandidates?: string[];
  isCustomTemplate?: boolean;
}

export const kimiProviderPresets: KimiProviderPreset[] = [
  {
    name: "Kimi",
    primePartner: true,
    websiteUrl: "https://platform.kimi.com?aff=cc-switch",
    apiKeyUrl: "https://platform.kimi.com?aff=cc-switch",
    settingsConfig: {
      provider_name: "kimi",
      type: "kimi",
      base_url: "https://api.moonshot.cn/v1",
      api_key: "",
      models: [{ id: "kimi-k2.7-code", name: "Kimi K2.7 Code" }],
    },
    category: "cn_official",
    icon: "kimi",
    iconColor: "#6366F1",
  },
  {
    name: "Kimi For Coding",
    primePartner: true,
    websiteUrl: "https://www.kimi.com/code/?aff=cc-switch",
    apiKeyUrl: "https://www.kimi.com/code/?aff=cc-switch",
    settingsConfig: {
      provider_name: "kimi_coding",
      type: "kimi",
      base_url: "https://api.kimi.com/coding/",
      api_key: "",
      models: [{ id: "kimi-for-coding", name: "Kimi For Coding" }],
    },
    category: "cn_official",
    icon: "kimi",
    iconColor: "#6366F1",
  },
];
