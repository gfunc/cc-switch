import { describe, it, expect } from "vitest";
import {
  validateToml,
  mcpServerToToml,
  tomlToMcpServer,
  extractIdFromToml,
} from "@/utils/tomlUtils";
import type { McpServerSpec } from "@/types";

describe("tomlUtils", () => {
  describe("validateToml", () => {
    it("returns empty string for valid TOML object", () => {
      expect(validateToml('command = "npx"')).toBe("");
    });

    it("returns empty string for empty input", () => {
      expect(validateToml("")).toBe("");
    });

    it("returns an error for top-level array input", () => {
      // TOML does not allow top-level arrays, so smol-toml reports a parse error
      const result = validateToml("[1, 2, 3]");
      expect(result).not.toBe("");
      expect(result).toContain("Invalid TOML");
    });

    it("returns an error message for malformed TOML", () => {
      const result = validateToml("command = ");
      expect(result).not.toBe("");
    });
  });

  describe("mcpServerToToml", () => {
    it("serializes a stdio server", () => {
      const server: McpServerSpec = {
        type: "stdio",
        command: "npx",
        args: ["-y", "@modelcontextprotocol/server-filesystem"],
      };
      const toml = mcpServerToToml(server);
      expect(toml).toContain('type = "stdio"');
      expect(toml).toContain('command = "npx"');
      expect(toml).toContain("-y");
    });

    it("strips undefined fields", () => {
      const server: McpServerSpec = {
        type: "http",
        url: "http://localhost:3000",
      };
      const toml = mcpServerToToml(server);
      expect(toml).not.toContain("args");
      expect(toml).not.toContain("env");
    });

    it("preserves unknown extension fields", () => {
      const server: McpServerSpec = {
        type: "stdio",
        command: "npx",
        timeout_ms: 30000,
      };
      const toml = mcpServerToToml(server);
      expect(toml).toContain("timeout_ms");
      expect(toml).toContain("30000");
    });
  });

  describe("tomlToMcpServer", () => {
    it("parses a direct stdio server config", () => {
      const server = tomlToMcpServer('type = "stdio"\ncommand = "npx"');
      expect(server.type).toBe("stdio");
      expect(server.command).toBe("npx");
    });

    it("parses [mcp_servers.id] format and takes the first server", () => {
      const toml = `
[mcp_servers.filesystem]
type = "stdio"
command = "npx"
args = ["/tmp"]
      `.trim();
      const server = tomlToMcpServer(toml);
      expect(server.type).toBe("stdio");
      expect(server.command).toBe("npx");
    });

    it("parses [mcp.servers.id] fallback format", () => {
      const toml = `
[mcp.servers.filesystem]
type = "http"
url = "http://example.com"
      `.trim();
      const server = tomlToMcpServer(toml);
      expect(server.type).toBe("http");
      expect(server.url).toBe("http://example.com");
    });

    it("throws for empty input", () => {
      expect(() => tomlToMcpServer("")).toThrow();
    });

    it("throws for stdio config missing command", () => {
      expect(() => tomlToMcpServer('type = "stdio"')).toThrow("command");
    });

    it("throws for http config missing url", () => {
      expect(() => tomlToMcpServer('type = "http"')).toThrow("url");
    });

    it("coerces args to strings", () => {
      const server = tomlToMcpServer('type = "stdio"\ncommand = "npx"\nargs = [123]');
      expect(server.args).toEqual(["123"]);
    });
  });

  describe("extractIdFromToml", () => {
    it("extracts id from [mcp_servers.id]", () => {
      expect(
        extractIdFromToml(`
[mcp_servers.filesystem]
type = "stdio"
command = "npx"
        `),
      ).toBe("filesystem");
    });

    it("extracts id from [mcp.servers.id]", () => {
      expect(
        extractIdFromToml(`
[mcp.servers.filesystem]
type = "stdio"
command = "npx"
        `),
      ).toBe("filesystem");
    });

    it("infers id from command basename", () => {
      expect(extractIdFromToml('command = "/usr/local/bin/my-server.js"')).toBe(
        "my-server",
      );
    });

    it("returns empty string for unparseable input", () => {
      expect(extractIdFromToml("not toml at all ===")).toBe("");
    });
  });
});
