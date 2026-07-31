import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useDebouncedValue } from "@/hooks/useDebouncedValue";
import { useLastValidValue } from "@/hooks/useLastValidValue";
import { useDarkMode } from "@/hooks/useDarkMode";

describe("generic hooks", () => {
  describe("useDebouncedValue", () => {
    beforeEach(() => {
      vi.useFakeTimers({ shouldAdvanceTime: true });
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it("returns initial value immediately", () => {
      const { result } = renderHook(() => useDebouncedValue("initial", 300));
      expect(result.current).toBe("initial");
    });

    it("updates value after delay", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useDebouncedValue(value, 300),
        { initialProps: { value: "a" } },
      );
      rerender({ value: "b" });
      expect(result.current).toBe("a");
      act(() => {
        vi.advanceTimersByTime(300);
      });
      expect(result.current).toBe("b");
    });

    it("resets timer on rapid changes", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useDebouncedValue(value, 300),
        { initialProps: { value: "a" } },
      );
      rerender({ value: "b" });
      act(() => {
        vi.advanceTimersByTime(200);
      });
      rerender({ value: "c" });
      act(() => {
        vi.advanceTimersByTime(200);
      });
      expect(result.current).toBe("a");
      act(() => {
        vi.advanceTimersByTime(100);
      });
      expect(result.current).toBe("c");
    });
  });

  describe("useLastValidValue", () => {
    it("returns current value when non-null", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useLastValidValue(value),
        { initialProps: { value: "first" } },
      );
      expect(result.current).toBe("first");
      rerender({ value: "second" });
      expect(result.current).toBe("second");
    });

    it("keeps last valid value when current becomes null", () => {
      const { result, rerender } = renderHook(
        ({ value }) => useLastValidValue(value),
        { initialProps: { value: "valid" as string | null | undefined } },
      );
      rerender({ value: null });
      expect(result.current).toBe("valid");
      rerender({ value: undefined });
      expect(result.current).toBe("valid");
    });

    it("returns null when no valid value has been seen", () => {
      const { result } = renderHook(() => useLastValidValue(null));
      expect(result.current).toBeNull();
    });
  });

  describe("useDarkMode", () => {
    beforeEach(() => {
      document.documentElement.classList.remove("dark");
    });

    afterEach(() => {
      document.documentElement.classList.remove("dark");
    });

    it("returns false when dark class is absent", () => {
      const { result } = renderHook(() => useDarkMode());
      expect(result.current).toBe(false);
    });

    it("returns true when dark class is present", () => {
      document.documentElement.classList.add("dark");
      const { result } = renderHook(() => useDarkMode());
      expect(result.current).toBe(true);
    });

    it("reacts to class changes", async () => {
      const { result } = renderHook(() => useDarkMode());
      expect(result.current).toBe(false);
      act(() => {
        document.documentElement.classList.add("dark");
      });
      await waitFor(() => expect(result.current).toBe(true));
    });
  });
});
