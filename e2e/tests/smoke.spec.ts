import { test, expect, type Page } from "@playwright/test";

const EMAIL = process.env.E2E_EMAIL ?? "elena@madespace.co";
const PASSWORD = process.env.E2E_PASSWORD ?? "changeme-dev-only";

async function login(page: Page) {
  await page.goto("/");
  await expect(page.getByText("Welcome back.")).toBeVisible();
  await page.getByLabel("Email address").fill(EMAIL);
  await page.getByLabel("Password").fill(PASSWORD);
  await page.getByRole("button", { name: /sign in/i }).click();
  // Lands on the Roles & Permissions page by default.
  await expect(page.getByRole("heading", { name: "Permission matrix" })).toBeVisible();
}

test("sign in and see the ACL matrix", async ({ page }) => {
  await login(page);
  await expect(page.getByRole("heading", { name: "Roles & Permissions" })).toBeVisible();
  await expect(page.getByText("deny-by-default")).toBeVisible();
});

test("cycling a matrix cell writes to the audit log", async ({ page }) => {
  await login(page);

  // Click the first editable (unlocked) cell in the "Invite user" row.
  const row = page.locator("tr.act-row", { hasText: "Invite user" });
  await row.locator("button.cellbtn:not(.locked)").first().click();
  // A confirmation toast appears.
  await expect(page.locator(".toast")).toBeVisible();

  // The change shows up in the Audit Log.
  await page.getByRole("button", { name: "Audit Log", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Audit Log" })).toBeVisible();
  await expect(page.locator("tbody").getByText("permission.edit").first()).toBeVisible();
});

test("create a custom role adds a new matrix column", async ({ page }) => {
  await login(page);
  await page.getByRole("button", { name: "New role" }).click();
  const name = `Stager ${Date.now().toString().slice(-5)}`;
  await page.getByPlaceholder("e.g. Stager, Auditor, Finance").fill(name);
  await page.getByRole("button", { name: "Create role" }).click();
  // The new role appears in the role rail.
  await expect(page.locator(".role-rail").getByText(name)).toBeVisible();
});

test("invite a user shows them in the roster", async ({ page }) => {
  await login(page);
  await page.getByRole("button", { name: "Users", exact: true }).click();
  await page.getByRole("button", { name: "Invite user" }).click();
  const email = `newhire${Date.now().toString().slice(-6)}@madespace.co`;
  await page.getByPlaceholder("name@madespace.co").fill(email);
  await page.getByRole("button", { name: "Send invitation" }).click();
  await expect(page.locator("tbody").getByText(email)).toBeVisible();
});

test("create an API key reveals the token once", async ({ page }) => {
  await login(page);
  await page.getByRole("button", { name: "API Keys", exact: true }).click();
  await page.getByRole("button", { name: "New key" }).click();
  await page.getByPlaceholder("e.g. Booking sync").fill("Booking sync");
  await page.locator(".checkrow", { hasText: "Read users" }).click();
  await page.getByRole("button", { name: "Generate key" }).click();
  // The one-time token is shown and starts with the live prefix.
  await expect(page.getByText(/ms_live_/)).toBeVisible();
});
