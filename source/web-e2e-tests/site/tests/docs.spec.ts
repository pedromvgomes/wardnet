import { test, expect } from "@playwright/test";

test.describe("docs page", () => {
  test("renders the Documentation heading", async ({ page }) => {
    await page.goto("/docs");
    await expect(page.getByRole("heading", { name: "Documentation" })).toBeVisible();
  });

  test("shows the Recommended section", async ({ page }) => {
    await page.goto("/docs");
    await expect(page.getByText(/recommended/i)).toBeVisible();
  });

  test("shows the All topics section", async ({ page }) => {
    await page.goto("/docs");
    await expect(page.getByRole("heading", { name: "All topics" })).toBeVisible();
  });

  test("clicking the Installation link navigates to the article page", async ({
    page,
  }) => {
    await page.goto("/docs");
    await page.getByRole("link", { name: "Installation" }).first().click();

    await expect(page).toHaveURL("/docs/installation");
  });

  test("docs article page renders content", async ({ page }) => {
    await page.goto("/docs/configuration");
    // The article page renders the markdown article — verify a heading is present.
    await expect(page.getByRole("heading").first()).toBeVisible();
  });
});
