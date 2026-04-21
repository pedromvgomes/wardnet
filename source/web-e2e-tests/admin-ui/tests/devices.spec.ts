import { test, expect } from "@playwright/test";

test.describe("devices page", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/devices");
  });

  test("renders the Devices heading", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Devices" })).toBeVisible();
  });

  test("renders Managed and Discovered tabs", async ({ page }) => {
    await expect(page.getByRole("tab", { name: "Managed" })).toBeVisible();
    await expect(page.getByRole("tab", { name: "Discovered" })).toBeVisible();
  });
});
