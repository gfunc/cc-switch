import { describe, it, expect } from "vitest";
import { formatJSON, parseSmartMcpJson } from "@/utils/formatters";

describe("formatters", () => {
  describe("formatJSON", () => {
    it("returns empty string for empty input", () => {
      expect(formatJSON("")).toBe("");
      expect(formatJSON("   ")).toBe("");
    });

    it("pretty-prints compact JSON", () => {
      expect(formatJSON('{"a":1,"b":2}')).toBe(
        JSON.stringify({ a: 1, b: 2 }, null, 2),
      );
    });

    it("throws on invalid JSON", () => {
      expect(() => formatJSON("not json")).toThrow();
    });
  });

  describe("parseSmartMcpJson", () => {
    it("returns empty config for empty input", () => {
      const result = parseSmartMcpJson("");
      expect(result.config).toEqual({});
      expect(result.formattedConfig).toBe("");
    });

    it("parses a bare server config object", () => {
      const result = parseSmartMcpJson('{"command":"npx","args":["-y"]}');
      expect(result.config).toEqual({ command: "npx", args: ["-y"] });
      expect(result.id).toBeUndefined();
    });

    it("extracts id from single-key wrapper object", () => {
      const result = parseSmartMcpJson(
        '{"filesystem":{"command":"npx","args":["/tmp"]}}',
      );
      expect(result.id).toBe("filesystem");
      expect(result.config).toEqual({ command: "npx", args: ["/tmp"] });
      expect(result.formattedConfig).toContain('"command": "npx"');
    });

    it("wraps a key-value fragment into a complete object", () => {
      const result = parseSmartMcpJson('"server": {"command": "npx"}');
      expect(result.id).toBe("server");
      expect(result.config).toEqual({ command: "npx" });
    });

    it("throws on invalid JSON", () => {
      expect(() => parseSmartMcpJson("not json")).toThrow();
    });
  });
});
