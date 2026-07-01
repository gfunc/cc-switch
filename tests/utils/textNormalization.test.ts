import { describe, it, expect } from "vitest";
import {
  normalizeQuotes,
  normalizeTomlText,
} from "@/utils/textNormalization";

describe("textNormalization", () => {
  describe("normalizeQuotes", () => {
    it("returns falsy input unchanged", () => {
      expect(normalizeQuotes("")).toBe("");
    });

    it("normalizes Chinese double quotes to ASCII", () => {
      expect(normalizeQuotes("“hello”")).toBe('"hello"');
      expect(normalizeQuotes("„hello‟")).toBe('"hello"');
      expect(normalizeQuotes("＂hello＂")).toBe('"hello"');
    });

    it("normalizes Chinese single quotes to ASCII", () => {
      expect(normalizeQuotes("‘hello’")).toBe("'hello'");
      expect(normalizeQuotes("＇hello＇")).toBe("'hello'");
    });

    it("leaves book name quotes alone", () => {
      expect(normalizeQuotes("《hello》")).toBe("《hello》");
      expect(normalizeQuotes("「hello」")).toBe("「hello」");
    });
  });

  describe("normalizeTomlText", () => {
    it("delegates to normalizeQuotes", () => {
      expect(normalizeTomlText("“key\" = \"value\"")).toBe(
        '"key" = "value"',
      );
    });
  });
});
