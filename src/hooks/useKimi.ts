import { useQuery, type QueryClient } from "@tanstack/react-query";
import { providersApi } from "@/lib/api/providers";

/**
 * Centralized query keys for all Kimi-related queries.
 */
export const kimiKeys = {
  all: ["kimi"] as const,
  liveProviderIds: ["kimi", "liveProviderIds"] as const,
};

/**
 * Invalidate Kimi caches that change when a provider is added/updated/deleted/switched.
 */
export function invalidateKimiProviderCaches(queryClient: QueryClient) {
  return queryClient.invalidateQueries({
    queryKey: kimiKeys.liveProviderIds,
  });
}

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
