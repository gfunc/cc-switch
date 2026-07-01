import { useQuery } from "@tanstack/react-query";
import { providersApi } from "@/lib/api/providers";

/**
 * Centralized query keys for all Kimi-related queries.
 */
export const kimiKeys = {
  all: ["kimi"] as const,
  liveProviderIds: ["kimi", "liveProviderIds"] as const,
};

/**
 * Fetch the provider names currently written to Kimi's live config.toml.
 */
export function useKimiLiveProviderIds(enabled: boolean) {
  return useQuery({
    queryKey: kimiKeys.liveProviderIds,
    queryFn: () => providersApi.getKimiLiveProviderIds(),
    enabled,
  });
}
