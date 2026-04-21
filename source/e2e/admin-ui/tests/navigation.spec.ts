import { test, expect } from "@playwright/test";

test.describe("navigation", () => {
  test("unknown route renders the 404 page", async ({ page }) => {
    await page.goto("/this-route-does-not-exist");
    await expect(page.getByText(/not found/i)).toBeVisible();
  });

  test("sidebar link navigates to Devices page", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "Devices" }).click();
    await expect(page).toHaveURL("/devices");
    await expect(page.getByRole("heading", { name: "Devices" })).toBeVisible();
  });

  test("sidebar link navigates to Tunnels page", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "Tunnels" }).click();
    await expect(page).toHaveURL("/tunnels");
    await expect(page.getByRole("heading", { name: "Tunnels" })).toBeVisible();
  });

  test("sidebar link navigates to DHCP page", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "DHCP" }).click();
    await expect(page).toHaveURL("/dhcp");
  });

  test("sidebar link navigates to Settings page", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "Settings" }).click();
    await expect(page).toHaveURL("/settings");
  });
});
