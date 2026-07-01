import { useState, useCallback, useMemo } from "react";
import type { AppId } from "@/lib/api";
import { useProvidersQuery } from "@/lib/query/queries";
import type {
  KimiModel,
  KimiProviderSettingsConfig,
} from "@/config/kimiProviderPresets";

interface UseKimiFormStateParams {
  initialData?: {
    settingsConfig?: Record<string, unknown>;
  };
  appId: AppId;
  providerId?: string;
  onSettingsConfigChange: (config: string) => void;
  getSettingsConfig: () => string;
}

const KIMI_DEFAULT_CONFIG_OBJ = {
  provider_name: "",
  type: "kimi",
  base_url: "",
  api_key: "",
} as const;

export const KIMI_DEFAULT_CONFIG = JSON.stringify(
  KIMI_DEFAULT_CONFIG_OBJ,
  null,
  2,
);

export interface KimiFormState {
  kimiProviderKey: string;
  setKimiProviderKey: (key: string) => void;
  kimiBaseUrl: string;
  kimiApiKey: string;
  kimiModels: KimiModel[];
  existingKimiKeys: string[];
  handleKimiBaseUrlChange: (baseUrl: string) => void;
  handleKimiApiKeyChange: (apiKey: string) => void;
  handleKimiModelsChange: (models: KimiModel[]) => void;
  resetKimiState: (config?: Partial<KimiProviderSettingsConfig>) => void;
}

function parseKimiField<T>(
  initialData: UseKimiFormStateParams["initialData"],
  field: string,
  fallback: T,
): T {
  try {
    if (initialData?.settingsConfig) {
      return (initialData.settingsConfig[field] as T) || fallback;
    }
    return (
      ((KIMI_DEFAULT_CONFIG_OBJ as Record<string, unknown>)[field] as T) ||
      fallback
    );
  } catch {
    return fallback;
  }
}

export function useKimiFormState({
  initialData,
  appId,
  providerId,
  onSettingsConfigChange,
  getSettingsConfig,
}: UseKimiFormStateParams): KimiFormState {
  const { data: kimiProvidersData } = useProvidersQuery("kimi");
  const existingKimiKeys = useMemo(() => {
    if (!kimiProvidersData?.providers) return [];
    return Object.keys(kimiProvidersData.providers).filter(
      (k) => k !== providerId,
    );
  }, [kimiProvidersData?.providers, providerId]);

  const [kimiProviderKey, _setKimiProviderKey] = useState<string>(() => {
    if (appId !== "kimi") return "";
    return providerId || "";
  });

  const [kimiBaseUrl, setKimiBaseUrl] = useState<string>(() => {
    if (appId !== "kimi") return "";
    return parseKimiField(initialData, "base_url", "");
  });

  const [kimiApiKey, setKimiApiKey] = useState<string>(() => {
    if (appId !== "kimi") return "";
    return parseKimiField(initialData, "api_key", "");
  });

  const [kimiModels, setKimiModels] = useState<KimiModel[]>(() => {
    if (appId !== "kimi") return [];
    return parseKimiField<KimiModel[]>(initialData, "models", []);
  });

  const updateKimiConfig = useCallback(
    (updater: (config: Record<string, unknown>) => void) => {
      try {
        const config = JSON.parse(getSettingsConfig() || KIMI_DEFAULT_CONFIG);
        updater(config);
        onSettingsConfigChange(JSON.stringify(config, null, 2));
      } catch {
        // ignore
      }
    },
    [getSettingsConfig, onSettingsConfigChange],
  );

  const handleKimiBaseUrlChange = useCallback(
    (baseUrl: string) => {
      setKimiBaseUrl(baseUrl);
      updateKimiConfig((config) => {
        config.base_url = baseUrl.trim().replace(/\/+$/, "");
      });
    },
    [updateKimiConfig],
  );

  const handleKimiApiKeyChange = useCallback(
    (apiKey: string) => {
      setKimiApiKey(apiKey);
      updateKimiConfig((config) => {
        config.api_key = apiKey;
      });
    },
    [updateKimiConfig],
  );

  const handleKimiModelsChange = useCallback(
    (models: KimiModel[]) => {
      setKimiModels(models);
      updateKimiConfig((config) => {
        if (models.length === 0) {
          delete config.models;
        } else {
          config.models = models;
        }
      });
    },
    [updateKimiConfig],
  );

  const setKimiProviderKey = useCallback(
    (key: string) => {
      _setKimiProviderKey(key);
      updateKimiConfig((config) => {
        config.provider_name = key;
      });
    },
    [updateKimiConfig],
  );

  const resetKimiState = useCallback(
    (config?: Partial<KimiProviderSettingsConfig>) => {
      _setKimiProviderKey("");
      setKimiBaseUrl(config?.base_url || "");
      setKimiApiKey(config?.api_key || "");
      setKimiModels(config?.models ?? []);
      // provider_name is synced to the provider key input in the form;
      // if a preset supplies one, keep it in config until the user edits the key.
      updateKimiConfig((cfg) => {
        if (config?.provider_name) {
          cfg.provider_name = config.provider_name;
        } else {
          cfg.provider_name = "";
        }
      });
    },
    [updateKimiConfig],
  );

  return {
    kimiProviderKey,
    setKimiProviderKey,
    kimiBaseUrl,
    kimiApiKey,
    kimiModels,
    existingKimiKeys,
    handleKimiBaseUrlChange,
    handleKimiApiKeyChange,
    handleKimiModelsChange,
    resetKimiState,
  };
}
