import { test, expect } from "@playwright/test";

// These tests run with the default project storage state (authenticated admin).
// The wizard form is still reachable at /setup after setup completes — submitting
// it returns a 409 from the API, which the UI surfaces as an error message.
// Client-side validation (password length, mismatch) is exercised without hitting
// the network at all.
test.describe("setup wizard", () => {
  test("form renders with username, password, and confirm-password fields", async ({
    page,
  }) => {
    await page.goto("/setup");

    await expect(page.getByRole("heading", { name: "Create Admin Account" })).toBeVisible();
    await expect(page.getByLabel("Username")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();
    await expect(page.getByLabel("Confirm Password")).toBeVisible();
    await expect(page.getByRole("button", { name: /create account/i })).toBeVisible();
  });

  test("shows error when password is shorter than 8 characters", async ({ page }) => {
    await page.goto("/setup");

    await page.getByLabel("Username").fill("admin");
    await page.getByLabel("Password").fill("short");
    await page.getByLabel("Confirm Password").fill("short");
    await page.getByRole("button", { name: /create account/i }).click();

    await expect(page.getByText("Password must be at least 8 characters.")).toBeVisible();
  });

  test("shows error when passwords do not match", async ({ page }) => {
    await page.goto("/setup");

    await page.getByLabel("Username").fill("admin");
    await page.getByLabel("Password").fill("password123");
    await page.getByLabel("Confirm Password").fill("different123");
    await page.getByRole("button", { name: /create account/i }).click();

    await expect(page.getByText("Passwords do not match.")).toBeVisible();
  });

  test("shows already-completed error when setup is submitted again", async ({
    page,
  }) => {
    await page.goto("/setup");

    await page.getByLabel("Username").fill("newadmin");
    await page.getByLabel("Password").fill("newpassword123");
    await page.getByLabel("Confirm Password").fill("newpassword123");
    await page.getByRole("button", { name: /create account/i }).click();

    await expect(page.getByText("Setup has already been completed.")).toBeVisible();
  });
});
