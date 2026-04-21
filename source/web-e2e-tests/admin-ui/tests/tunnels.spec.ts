import { test, expect } from "@playwright/test";

test.describe("tunnels page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/tunnels");
  });

  test("renders the Tunnels heading", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Tunnels" })).toBeVisible();
  });

  test("shows the Add Tunnel button or empty-state action", async ({ page }) => {
    // Either the header button (when tunnels exist) or the empty-state button.
    const addButton = page.getByRole("button", { name: "Add Tunnel" });
    await expect(addButton.first()).toBeVisible();
  });

  test("clicking Add Tunnel opens the create tunnel sheet", async ({ page }) => {
    await page.getByRole("button", { name: "Add Tunnel" }).first().click();

    // The sheet opens — verify the Manual and Provider tabs are present.
    await expect(page.getByRole("tab", { name: "Manual" })).toBeVisible();
    await expect(page.getByRole("tab", { name: "Provider" })).toBeVisible();
  });
});
