import { test, expect } from "@playwright/test";

test.describe("navigation", () => {
  test("unknown route renders the 404 page", async ({ page }) => {
    await page.goto("/this-route-does-not-exist");
    await expect(page.getByText(/not found/i)).toBeVisible();
  });

  test("navbar Docs link navigates to /docs", async ({ page }) => {
    await page.goto("/?view=content");

    await page.getByRole("link", { name: "Docs" }).click();
    await expect(page).toHaveURL("/docs");
    await expect(page.getByRole("heading", { name: "Documentation" })).toBeVisible();
  });

  test("docs back link returns to /?view=content", async ({ page }) => {
    await page.goto("/docs");

    // The Navbar in docs mode has a back link.
    await page.getByRole("link", { name: /wardnet/i }).click();
    await expect(page).toHaveURL("/?view=content");
  });
});
