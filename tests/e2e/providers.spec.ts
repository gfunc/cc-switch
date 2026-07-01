import { test, expect } from "@playwright/test";

const STATIC_TOKEN = "e2e-test-token";
const APP = "claude";

async function authToken(request: import("@playwright/test").APIRequestContext) {
  const res = await request.post("/api/v1/auth/generate", {
    headers: { Authorization: `Bearer ${STATIC_TOKEN}` },
  });
  expect(res.ok()).toBe(true);
  const body = await res.json();
  expect(body.success).toBe(true);
  return body.data as string;
}

async function seedProvider(
  request: import("@playwright/test").APIRequestContext,
  token: string,
  id: string,
  name: string,
  baseUrl: string,
  sortIndex: number,
) {
  const res = await request.post("/api/v1/providers", {
    headers: { Authorization: `Bearer ${token}` },
    data: {
      provider: {
        id,
        name,
        settingsConfig: { env: { ANTHROPIC_BASE_URL: baseUrl } },
        category: "custom",
        sortIndex,
        createdAt: Date.now(),
      },
      app: APP,
    },
  });
  expect(res.ok()).toBe(true);
}

async function cleanupProvider(
  request: import("@playwright/test").APIRequestContext,
  token: string,
  id: string,
) {
  // Ignore failures (e.g., provider does not exist) so cleanup is idempotent.
  await request.delete(`/api/v1/providers/${id}?app=${APP}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
}

test.describe("provider management (web)", () => {
  test("switches and deletes a provider from the UI", async ({ page, request }) => {
    const token = await authToken(request);

    // Remove stale providers from previous runs so the test starts from a
    // deterministic state even when the E2E database is reused.
    await cleanupProvider(request, token, "e2e-switch-provider");
    await cleanupProvider(request, token, "e2e-delete-provider");

    // Seed two providers: one to switch to and one to delete.
    // The app prevents deleting the provider that is currently in use,
    // so we switch to one and delete the other.
    await seedProvider(
      request,
      token,
      "e2e-switch-provider",
      "E2E Switch Provider",
      "http://localhost:9999",
      0,
    );
    await seedProvider(
      request,
      token,
      "e2e-delete-provider",
      "E2E Delete Provider",
      "http://localhost:9998",
      1,
    );

    // Log in through the UI.
    await page.goto("/");
    await page.fill('input[id="token"]', token);
    await page.click('button[type="submit"]');
    await expect(page.getByRole("banner")).toBeVisible();

    // Dismiss first-run welcome dialog if it appears.
    const gotItButton = page.getByRole("button", { name: /Got it|知道了/i });
    let hasWelcomeDialog = false;
    try {
      await gotItButton.waitFor({ state: "visible", timeout: 5000 });
      hasWelcomeDialog = true;
    } catch (error) {
      // Only ignore a timeout when the dialog did not appear.
      // Re-throw anything else so real UI failures surface.
      if (!(error instanceof Error && error.name === "TimeoutError")) {
        throw error;
      }
    }
    if (hasWelcomeDialog) {
      await gotItButton.click();
      await expect(gotItButton).toBeHidden();
    }

    // Ensure we are on the Claude providers view.
    const claudeTab = page.getByRole("button", { name: /Claude Code$/i });
    await expect(claudeTab).toBeVisible();
    await claudeTab.click();

    // The seeded providers should be visible.
    const switchHeading = page.getByRole("heading", {
      name: "E2E Switch Provider",
    });
    const deleteHeading = page.getByRole("heading", {
      name: "E2E Delete Provider",
    });
    await expect(switchHeading).toBeVisible();
    await expect(deleteHeading).toBeVisible();

    // Hover the switch provider card to reveal action buttons, then switch to it.
    // ProviderCard.tsx uses the `bg-card` Tailwind class on the card root, so we
    // scope the heading to that card to avoid matching nested headings.
    const switchCard = page.locator(".bg-card").filter({ has: switchHeading });
    await switchCard.hover();
    const enableButton = switchCard.getByRole("button", { name: /Enable|启用/i });
    await expect(enableButton).toBeVisible();
    await enableButton.click();

    // After switching, the main action should indicate it is in use.
    await expect(switchCard.getByRole("button", { name: /In Use|已在用/i })).toBeVisible();

    // Hover the delete provider card and delete it.
    // ProviderCard.tsx uses the `bg-card` Tailwind class on the card root.
    const deleteCard = page.locator(".bg-card").filter({ has: deleteHeading });
    await deleteCard.hover();
    const deleteButton = deleteCard.getByRole("button", { name: /Delete|删除/i });
    await expect(deleteButton).toBeVisible();
    await deleteButton.click();

    // Confirm deletion in the delete-confirmation dialog.
    const confirmButton = page
      .getByRole("dialog", { name: /Delete Provider|删除供应商/i })
      .getByRole("button", { name: /Confirm|确认/i });
    await expect(confirmButton).toBeVisible();
    await confirmButton.click();

    // The deleted provider should disappear from the list.
    await expect(deleteHeading).toHaveCount(0);

    // The switched-to provider should still be present and in use.
    await expect(switchHeading).toBeVisible();
    await expect(switchCard.getByRole("button", { name: /In Use|已在用/i })).toBeVisible();
  });
});
