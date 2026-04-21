import path from "path";
import { defineConfig, devices } from "@playwright/test";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const ADMIN_UI_URL = "http://127.0.0.1:7411";
const SITE_URL = "http://localhost:7413";

// CI supplies the path to a pre-built binary; local dev falls back to `cargo run`.
const mockBin = process.env.WARDNETD_MOCK_BIN;
const mockCommand = mockBin
  ? `${mockBin} --no-seed --no-events`
  : "cargo run -p wardnetd-mock -- --no-seed --no-events";
const mockCwd = mockBin
  ? path.resolve(__dirname)
  : path.resolve(__dirname, "../daemon");

export default defineConfig({
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: [["html"], ["list"]],

  projects: [
    // ----- Admin UI -----
    // globalSetup creates the admin account and persists the session cookie so
    // all tests in this project start already authenticated.
    {
      name: "admin-ui",
      testDir: "./admin-ui/tests",
      testMatch: "**/*.spec.ts",
      globalSetup: "./admin-ui/global-setup.ts",
      use: {
        ...devices["Desktop Chrome"],
        baseURL: ADMIN_UI_URL,
        storageState: "./admin-ui/.auth/admin.json",
      },
    },

    // ----- Site -----
    {
      name: "site",
      testDir: "./site/tests",
      testMatch: "**/*.spec.ts",
      use: {
        ...devices["Desktop Chrome"],
        baseURL: SITE_URL,
      },
    },
  ],

  webServer: [
    {
      command: mockCommand,
      cwd: mockCwd,
      url: `${ADMIN_UI_URL}/api/info`,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
    },
    {
      command: "yarn dev",
      cwd: path.resolve(__dirname, "../site"),
      url: SITE_URL,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
