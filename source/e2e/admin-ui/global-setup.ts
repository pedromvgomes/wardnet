import { chromium } from "@playwright/test";
import path from "path";
import { fileURLToPath } from "url";
import fs from "fs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const BASE_URL = "http://127.0.0.1:7411";
const USERNAME = "admin";
const PASSWORD = "testpassword123";
const AUTH_FILE = path.join(__dirname, ".auth/admin.json");

/**
 * Creates the admin account (if not already done) and saves the session
 * cookie to disk so every test in the admin-ui project starts authenticated.
 */
export default async function globalSetup() {
  fs.mkdirSync(path.dirname(AUTH_FILE), { recursive: true });

  const browser = await chromium.launch();
  const context = await browser.newContext();

  // Create the admin account — 409 means it was already created (idempotent).
  const setupRes = await context.request.post(`${BASE_URL}/api/setup`, {
    data: { username: USERNAME, password: PASSWORD },
  });
  if (!setupRes.ok() && setupRes.status() !== 409) {
    const body = await setupRes.text();
    throw new Error(`Setup failed (${setupRes.status()}): ${body}`);
  }

  // Login to get a session cookie.
  const loginRes = await context.request.post(`${BASE_URL}/api/auth/login`, {
    data: { username: USERNAME, password: PASSWORD },
  });
  if (!loginRes.ok()) {
    const body = await loginRes.text();
    throw new Error(`Login failed (${loginRes.status()}): ${body}`);
  }

  await context.storageState({ path: AUTH_FILE });
  await browser.close();
}
