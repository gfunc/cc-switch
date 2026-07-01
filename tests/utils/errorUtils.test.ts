import { describe, it, expect, vi } from "vitest";
import {
  extractErrorMessage,
  translateMcpBackendError,
} from "@/utils/errorUtils";

const mockT = vi.fn((key: string, _opts?: any) => key);

describe("errorUtils", () => {
  describe("extractErrorMessage", () => {
    it("returns empty string for null/undefined/falsy errors", () => {
      expect(extractErrorMessage(null)).toBe("");
      expect(extractErrorMessage(undefined)).toBe("");
      expect(extractErrorMessage("")).toBe("");
    });

    it("returns string errors as-is", () => {
      expect(extractErrorMessage("something broke")).toBe("something broke");
    });

    it("extracts Error.message", () => {
      expect(extractErrorMessage(new Error("nested error"))).toBe(
        "nested error",
      );
    });

    it("extracts message field from object", () => {
      expect(extractErrorMessage({ message: "object message" })).toBe(
        "object message",
      );
    });

    it("falls back to error/detail field", () => {
      expect(extractErrorMessage({ error: "error value" })).toBe("error value");
      expect(extractErrorMessage({ detail: "detail value" })).toBe(
        "detail value",
      );
    });

    it("extracts message from nested payload", () => {
      expect(
        extractErrorMessage({ payload: { message: "payload message" } }),
      ).toBe("payload message");
    });

    it("returns empty string when nothing matches", () => {
      expect(extractErrorMessage({ code: 500 })).toBe("");
    });
  });

  describe("translateMcpBackendError", () => {
    it("returns empty string for empty input", () => {
      expect(translateMcpBackendError("", mockT)).toBe("");
    });

    it("maps id required error", () => {
      expect(
        translateMcpBackendError("MCP 服务器 ID 不能为空", mockT),
      ).toBe("mcp.error.idRequired");
    });

    it("maps command required error", () => {
      expect(
        translateMcpBackendError("stdio 类型的 MCP 服务器缺少 command 字段", mockT),
      ).toBe("mcp.error.commandRequired");
    });

    it("maps url required error", () => {
      expect(
        translateMcpBackendError("http 类型的 MCP 服务器缺少 url 字段", mockT),
      ).toBe("mcp.wizard.urlRequired");
    });

    it("maps JSON invalid errors", () => {
      expect(
        translateMcpBackendError("MCP 服务器定义必须为 JSON 对象", mockT),
      ).toBe("mcp.error.jsonInvalid");
      expect(
        translateMcpBackendError("MCP 服务器 name 必须为字符串", mockT),
      ).toBe("mcp.error.jsonInvalid");
    });

    it("maps TOML invalid errors", () => {
      expect(
        translateMcpBackendError("解析 config.toml 失败", mockT),
      ).toBe("mcp.error.tomlInvalid");
      expect(
        translateMcpBackendError("无法识别的 TOML 格式", mockT),
      ).toBe("mcp.error.tomlInvalid");
    });

    it("returns empty string for unknown messages", () => {
      expect(translateMcpBackendError("random unknown text", mockT)).toBe("");
    });
  });
});
