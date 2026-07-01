import { describe, it, expect } from "vitest";
import {
  resolveUsageRange,
  getUsageRangePresetLabel,
} from "@/lib/usageRange";
import type { UsageRangeSelection } from "@/types/usage";

describe("usageRange", () => {
  describe("resolveUsageRange", () => {
    const nowMs = new Date("2026-01-15T12:00:00.000Z").getTime();

    it("resolves today preset to start of local day", () => {
      const selection: UsageRangeSelection = { preset: "today" };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.endDate).toBe(Math.floor(nowMs / 1000));
      expect(result.startDate).toBeLessThanOrEqual(result.endDate);
    });

    it("resolves 1d preset to last 24 hours", () => {
      const selection: UsageRangeSelection = { preset: "1d" };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.endDate - result.startDate).toBe(24 * 60 * 60);
    });

    it("resolves 7d preset to 7 days lookback", () => {
      const selection: UsageRangeSelection = { preset: "7d" };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.endDate - result.startDate).toBeGreaterThanOrEqual(
        6 * 24 * 60 * 60,
      );
      expect(result.endDate - result.startDate).toBeLessThanOrEqual(
        7 * 24 * 60 * 60,
      );
    });

    it("resolves custom preset with explicit dates", () => {
      const selection: UsageRangeSelection = {
        preset: "custom",
        customStartDate: 1700000000,
        customEndDate: 1700100000,
      };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.startDate).toBe(1700000000);
      expect(result.endDate).toBe(1700100000);
    });

    it("resolves custom preset with liveEndTime to now", () => {
      const selection: UsageRangeSelection = {
        preset: "custom",
        customStartDate: 1700000000,
        liveEndTime: true,
      };
      const result = resolveUsageRange(selection, nowMs);
      expect(result.startDate).toBe(1700000000);
      expect(result.endDate).toBe(Math.floor(nowMs / 1000));
    });
  });

  describe("getUsageRangePresetLabel", () => {
    const t = (key: string, opts?: { defaultValue?: string }) =>
      opts?.defaultValue ?? key;

    it("returns labels for all presets", () => {
      expect(getUsageRangePresetLabel("today", t)).toBe("当天");
      expect(getUsageRangePresetLabel("1d", t)).toBe("1d");
      expect(getUsageRangePresetLabel("7d", t)).toBe("7d");
      expect(getUsageRangePresetLabel("14d", t)).toBe("14d");
      expect(getUsageRangePresetLabel("30d", t)).toBe("30d");
      expect(getUsageRangePresetLabel("custom", t)).toBe("日历筛选");
    });
  });
});
