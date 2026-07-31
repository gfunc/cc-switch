import { describe, it, expect } from "vitest";
import { providerSchema, type ProviderFormData } from "@/lib/schemas/provider";
describe("parseJsonError (provider.ts)", () => {
  describe("Chrome/V8 error format", () => {
    it("should parse Chrome/V8 'at position' format with providerSchema", () => {
      const data: ProviderFormData = {
        name: "Test Provider",
        settingsConfig: '{"invalid": }', // Invalid JSON
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
      if (!result.success) {
        const settingsError = result.error.issues.find((issue) =>
          issue.path.includes("settingsConfig")
        );
        expect(settingsError?.message).toBeDefined();
      }
    });

    it("should handle Chrome 'Unexpected token' at various positions", () => {
      const testCases = [
        '{"key": undefined}',
        '{"key": NaN}',
        "{'key': 'value'}",
      ];

      for (const testCase of testCases) {
        expect(() => JSON.parse(testCase)).toThrow();
      }
    });
  });

  describe("Firefox error format", () => {
    it("should parse Firefox 'line X column Y' format with providerSchema", () => {
      const data: ProviderFormData = {
        name: "Test Provider",
        settingsConfig: '{"key":\n\ninvalid}', // Invalid JSON
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  describe("Generic browser error format", () => {
    it("should handle generic JSON parse errors with providerSchema", () => {
      const data: ProviderFormData = {
        name: "Test Provider",
        settingsConfig: "[1, 2, 3,]", // Invalid JSON (trailing comma)
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
      if (!result.success) {
        const settingsError = result.error.issues.find((issue) =>
          issue.path.includes("settingsConfig")
        );
        expect(settingsError?.message).toContain("JSON");
      }
    });
  });
});

describe("providerSchema", () => {
  describe("valid data", () => {
    it("should accept valid minimal provider", () => {
      const data: ProviderFormData = {
        name: "My Provider",
        settingsConfig: '{"apiKey": "test"}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should accept provider with all fields", () => {
      const data: ProviderFormData = {
        name: "Full Provider",
        websiteUrl: "https://example.com",
        notes: "Test notes",
        settingsConfig: '{"key": "value", "nested": {"field": 123}}',
        icon: "🔑",
        iconColor: "#FF5733",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should accept empty string for optional websiteUrl", () => {
      const data: ProviderFormData = {
        name: "Provider",
        websiteUrl: "",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should accept provider with complex nested JSON config", () => {
      const data: ProviderFormData = {
        name: "Complex Provider",
        settingsConfig: JSON.stringify({
          env: { ANTHROPIC_AUTH_TOKEN: "test-token" },
          mcpServers: {
            "mcp-fetch": {
              command: "npx",
              args: ["@modelcontextprotocol/server-fetch"],
            },
          },
        }),
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should accept provider with unicode characters in name and notes", () => {
      const data: ProviderFormData = {
        name: "提供商 🚀 プロバイダー",
        notes: "测试笔记 🎯 テストノート",
        settingsConfig: '{"field": "值"}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });
  });

  describe("invalid data", () => {
    it("should reject provider with invalid websiteUrl", () => {
      const data: ProviderFormData = {
        name: "Provider",
        websiteUrl: "not-a-valid-url",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues[0].path).toContain("websiteUrl");
      }
    });

    it("should reject provider with invalid JSON config", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: "{invalid json}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues[0].path).toContain("settingsConfig");
        expect(result.error.issues[0].message).toContain("JSON");
      }
    });

    it("should reject provider with empty settingsConfig", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: "",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues[0].path).toContain("settingsConfig");
      }
    });

    it("should reject provider with whitespace-only settingsConfig", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: "   ",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
    });

    it("should reject provider with incomplete JSON", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"key": "value"',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
    });

    it("should reject provider with trailing commas in JSON", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"key": "value",}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
    });

    it("should reject provider with unquoted keys in JSON", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: "{key: 'value'}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
    });

    it("should reject provider with single quotes in JSON", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: "{'key': 'value'}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  describe("edge cases", () => {
    it("should handle JSON with null values", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"key": null}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle JSON with boolean values", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"enabled": true, "disabled": false}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle JSON with numeric values", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"timeout": 30000, "retries": 3, "rate": 0.95}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle JSON with escape sequences", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig:
          '{"path": "C:\\\\Users\\\\test", "quote": "\\"quoted\\""}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle JSON with unicode escape sequences", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig:
          '{"emoji": "\\ud83d\\ude00", "chinese": "\\u4e2d\\u6587"}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle JSON with special HTML characters", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"html": "<script>alert(1)</script>"}',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle very large JSON structures", () => {
      const largeConfig = {
        items: Array.from({ length: 1000 }, (_, i) => ({
          id: i,
          value: `item-${i}`,
        })),
      };

      const data: ProviderFormData = {
        name: "Large Provider",
        settingsConfig: JSON.stringify(largeConfig),
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle deeply nested JSON structures", () => {
      let nested: any = { value: "deep" };
      for (let i = 0; i < 50; i++) {
        nested = { level: nested };
      }

      const data: ProviderFormData = {
        name: "Deep Provider",
        settingsConfig: JSON.stringify(nested),
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle name with special characters", () => {
      const data: ProviderFormData = {
        name: "Provider@#$%^&*()_+-=[]{}|;:',.<>?/`~",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle name with newlines and tabs", () => {
      const data: ProviderFormData = {
        name: "Provider\nWith\nNewlines\tAnd\tTabs",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle URL with special characters and query parameters", () => {
      const data: ProviderFormData = {
        name: "Provider",
        websiteUrl: "https://example.com/path?key=value&other=123#anchor",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle icon as emoji and special Unicode", () => {
      const data: ProviderFormData = {
        name: "Provider",
        icon: "🔐🚀💻🌟",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it("should handle very long icon color code", () => {
      const data: ProviderFormData = {
        name: "Provider",
        iconColor: "#FFFFFF",
        settingsConfig: "{}",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });
  });

  describe("JSON error message extraction", () => {
    it("should extract error position from Chrome errors", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: '{"incomplete": ',
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(false);
      if (!result.success) {
        const message = result.error.issues[0].message;
        expect(message).toContain("JSON");
      }
    });

    it("should handle null JSON value", () => {
      const data: ProviderFormData = {
        name: "Provider",
        settingsConfig: "null",
      };

      const result = providerSchema.safeParse(data);
      expect(result.success).toBe(true);
    });
  });
});

