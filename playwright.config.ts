import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./browser-tests",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: "http://127.0.0.1:8081",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "node ./scripts/static-spa-server.mjs --port 8081",
    reuseExistingServer: false,
    timeout: 180_000,
    url: "http://127.0.0.1:8081",
  },
});
