import { defineConfig, devices } from "@playwright/test";

const API = "http://localhost:8080";
const WEB = "http://localhost:5173";

const DATABASE_URL =
  process.env.DATABASE_URL ?? "postgres://core:core@localhost:5440/core?sslmode=disable";
const SEED_OWNER_EMAIL = process.env.SEED_OWNER_EMAIL ?? "elena@madespace.co";
const SEED_OWNER_PASSWORD = process.env.SEED_OWNER_PASSWORD ?? "changeme-dev-only";

// Surface the seed credentials to the spec.
process.env.E2E_EMAIL = SEED_OWNER_EMAIL;
process.env.E2E_PASSWORD = SEED_OWNER_PASSWORD;

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  retries: 0,
  reporter: [["list"]],
  use: { baseURL: WEB, trace: "on-first-retry" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  // Bring up the API (which seeds the Owner on first boot) and the SPA. Requires
  // Postgres to be running first (`pnpm db:up` from the repo root).
  webServer: [
    {
      command: "cargo run -p core-web",
      cwd: "../apps/api",
      url: `${API}/api/health`,
      reuseExistingServer: true,
      timeout: 180_000,
      env: {
        DATABASE_URL,
        SEED_OWNER: "true",
        SEED_OWNER_EMAIL,
        SEED_OWNER_PASSWORD,
        API_BIND_ADDR: "127.0.0.1:8080",
        WEB_ORIGIN: WEB,
      },
    },
    {
      command: "pnpm --filter core dev",
      cwd: "..",
      url: WEB,
      reuseExistingServer: true,
      timeout: 60_000,
    },
  ],
});
