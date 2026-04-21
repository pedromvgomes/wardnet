import { test, expect } from "@playwright/test";

test.describe("home page", () => {
  test("renders the hero section by default", async ({ page }) => {
    await page.goto("/");
    // The hero is the default view — the content sections are hidden behind
    // the Explore button.
    await expect(page.getByRole("main")).toBeVisible();
  });

  test("clicking Explore reveals the Features section", async ({ page }) => {
    await page.goto("/");

    // Find and click the Explore / Get started CTA in the hero.
    await page.getByRole("button", { name: /explore/i }).click();

    await expect(page.getByRole("heading", { name: /features/i })).toBeVisible();
  });

  test("?view=content skips the hero and shows content directly", async ({ page }) => {
    await page.goto("/?view=content");

    // Content sections should be visible immediately.
    await expect(page.getByRole("heading", { name: /features/i })).toBeVisible();
  });

  test("navbar is visible in content view", async ({ page }) => {
    await page.goto("/?view=content");
    await expect(page.getByRole("navigation")).toBeVisible();
  });
});
