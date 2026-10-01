import { defineConfig } from "@playwright/test";
import { fileURLToPath } from "node:url";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 30_000,
  expect: { timeout: 10_000 },
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:5173",
    browserName: "chromium",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  webServer: [
    {
      name: "local-api",
      command: "cargo run -p local-stack-proof-api",
      cwd: fileURLToPath(new URL("../../", import.meta.url)),
      url: "http://127.0.0.1:3001/api/health",
      env: {
        API_BIND_ADDRESS: "127.0.0.1:3001",
        DATABASE_URL:
          "postgresql://local_stack_proof:local_stack_proof@127.0.0.1:54329/local_stack_proof",
      },
      reuseExistingServer: false,
      timeout: 120_000,
    },
    {
      name: "local-ui",
      command: "npm run dev -- --port 5173 --strictPort",
      url: "http://127.0.0.1:5173",
      env: { VITE_API_BASE_URL: "http://127.0.0.1:3001" },
      reuseExistingServer: false,
      timeout: 60_000,
    },
  ],
});
