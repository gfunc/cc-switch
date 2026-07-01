import { test, expect } from "@playwright/test";

const STATIC_TOKEN = "e2e-test-token";

async function authToken(request: import("@playwright/test").APIRequestContext) {
  const res = await request.post("/api/v1/auth/generate", {
    headers: { Authorization: `Bearer ${STATIC_TOKEN}` },
  });
  expect(res.ok()).toBe(true);
  const body = await res.json();
  expect(body.success).toBe(true);
  return body.data as string;
}

test.describe("settings (web)", () => {
  test("changes interface language from the settings page", async ({ page, request }) => {
    const token = await authToken(request);

    await page.goto("/");
    await page.fill('input[id="token"]', token);
    await page.click('button[type="submit"]');
    await expect(page.getByRole("banner")).toBeVisible();

    const gotItButton = page.getByRole("button", { name: /Got it|知道了/i });
    try {
      await gotItButton.waitFor({ state: "visible", timeout: 5000 });
      await gotItButton.click();
      await expect(gotItButton).toBeHidden();
    } catch {
      // No first-run dialog; proceed.
    }

    // Open settings from the header.
    await page.getByRole("button", { name: "Settings" }).click();

    // Wait for the settings page heading.
    const settingsHeading = page.getByRole("heading", { name: /Settings|设置/i });
    await expect(settingsHeading).toBeVisible();

    // The language section header is translated.
    const languageHeader = page.getByRole("heading", { name: /Language|语言/i });
    await expect(languageHeader).toBeVisible();

    // Switch to English.
    await page.getByRole("button", { name: "English" }).click();

    // After switching, the language section header should read "Language".
    await expect(page.getByRole("heading", { name: "Language" })).toBeVisible();

    // The settings heading should be in English.
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  });
});
