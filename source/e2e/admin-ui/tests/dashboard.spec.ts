import { test, expect } from "@playwright/test";

test.describe("dashboard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("renders the Dashboard heading", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  });

  test("renders stat cards for Devices and Tunnels", async ({ page }) => {
    await expect(page.getByText("Devices")).toBeVisible();
    await expect(page.getByText("Tunnels")).toBeVisible();
  });

  test("sidebar shows admin navigation links", async ({ page }) => {
    await expect(page.getByRole("link", { name: "Dashboard" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Devices" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Tunnels" })).toBeVisible();
    await expect(page.getByRole("link", { name: "DHCP" })).toBeVisible();
    await expect(page.getByRole("link", { name: "DNS" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Ad Blocking" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Settings" })).toBeVisible();
  });

  test("clicking the Devices stat card navigates to /devices", async ({ page }) => {
    await page.getByRole("link", { name: "Devices" }).first().click();
    await expect(page).toHaveURL("/devices");
  });
});
