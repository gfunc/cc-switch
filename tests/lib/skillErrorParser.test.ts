import { describe, it, expect, vi } from "vitest";
import {
  parseSkillError,
  formatSkillError,
} from "@/lib/errors/skillErrorParser";

describe("skillErrorParser", () => {
  describe("parseSkillError", () => {
    it("parses a structured JSON error", () => {
      const error = JSON.stringify({
        code: "SKILL_NOT_FOUND",
        context: { skillName: "missing" },
      });
      expect(parseSkillError(error)).toEqual({
        code: "SKILL_NOT_FOUND",
        context: { skillName: "missing" },
      });
    });

    it("returns null for plain text errors", () => {
      expect(parseSkillError("plain text error")).toBeNull();
    });

    it("returns null for invalid JSON", () => {
      expect(parseSkillError("{not json")).toBeNull();
    });

    it("returns null when code/context are missing", () => {
      expect(parseSkillError(JSON.stringify({ foo: "bar" }))).toBeNull();
    });
  });

  describe("formatSkillError", () => {
    const t = vi.fn(
      (key: string, _opts?: any) => key,
    ) as unknown as Parameters<typeof formatSkillError>[1];

    it("formats structured errors with i18n keys", () => {
      const error = JSON.stringify({
        code: "SKILL_NOT_FOUND",
        context: { skillName: "missing" },
      });
      const result = formatSkillError(error, t, "custom.title");
      expect(result.title).toBe("custom.title");
      expect(result.description).toContain("skills.error.skillNotFound");
    });

    it("appends suggestion when provided", () => {
      const error = JSON.stringify({
        code: "DOWNLOAD_FAILED",
        context: {},
        suggestion: "checkNetwork",
      });
      const result = formatSkillError(error, t);
      expect(result.description).toContain("skills.error.downloadFailed");
      expect(result.description).toContain("skills.error.suggestion.checkNetwork");
    });

    it("falls back to raw string for unstructured errors", () => {
      const result = formatSkillError("raw error", t);
      expect(result.title).toBe("skills.installFailed");
      expect(result.description).toBe("raw error");
    });

    it("uses common.error fallback for empty unstructured errors", () => {
      const result = formatSkillError("", t);
      expect(result.description).toBe("common.error");
    });
  });
});
