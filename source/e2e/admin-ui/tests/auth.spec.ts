import { test, expect } from "@playwright/test";

const USERNAME = "admin";
const PASSWORD = "testpassword123";

test.describe("unauthenticated", () => {
  // Start each test in this group with a clean (unauthenticated) browser context.
  test.use({ storageState: { cookies: [], origins: [] } });

  test("navigating to /login shows the login form", async ({ page }) => {
    await page.goto("/login");

    await expect(page.getByLabel("Username")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();
    await expect(page.getByRole("button", { name: /log in/i })).toBeVisible();
  });

  test("login with wrong credentials shows an error", async ({ page }) => {
    await page.goto("/login");

    await page.getByLabel("Username").fill(USERNAME);
    await page.getByLabel("Password").fill("wrongpassword");
    await page.getByRole("button", { name: /log in/i }).click();

    await expect(page.getByText("Invalid username or password.")).toBeVisible();
  });

  test("successful login redirects to the dashboard", async ({ page }) => {
    await page.goto("/login");

    await page.getByLabel("Username").fill(USERNAME);
    await page.getByLabel("Password").fill(PASSWORD);
    await page.getByRole("button", { name: /log in/i }).click();

    await expect(page).toHaveURL("/");
    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  });

  test("navigating to /devices while unauthenticated redirects to /login", async ({
    page,
  }) => {
    await page.goto("/devices");
    await expect(page).toHaveURL("/login");
  });
});

test.describe("authenticated", () => {
  test("sign out redirects to the home page as unauthenticated user", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();

    await page.getByRole("button", { name: /sign out/i }).click();

    // After logout the home route renders MyDevice (self-service view) or
    // redirects — either way the Dashboard heading must be gone.
    await expect(page.getByRole("heading", { name: "Dashboard" })).not.toBeVisible();
  });
});
