import { afterEach, describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../test/server";
import { meOwner, org } from "../test/fixtures";
import { App } from "../App";

const unauthorized = () =>
  new HttpResponse(JSON.stringify({ error: "unauthorized", message: "no" }), {
    status: 401,
    headers: { "content-type": "application/json" },
  });

function gotoReset(token: string) {
  window.history.replaceState(null, "", `/reset-password?token=${token}`);
}

describe("ResetPasswordPage", () => {
  afterEach(() => window.history.replaceState(null, "", "/"));

  it("sets a new password via the token and lands in the app shell", async () => {
    let done = false;
    gotoReset("reset-tok-1");
    server.use(
      http.get("/api/auth/me", () => (done ? HttpResponse.json(meOwner) : unauthorized())),
      http.get("/api/org", () => HttpResponse.json(org)),
      http.get("/api/permissions/matrix", () => HttpResponse.json({ groups: [], columns: [], cells: [] })),
      http.get("/api/roles", () => HttpResponse.json([])),
      http.post("/api/auth/reset-password", async ({ request }) => {
        const body = (await request.json()) as { token: string; password: string };
        if (body.token === "reset-tok-1" && body.password.length >= 12) {
          done = true;
          return HttpResponse.json(meOwner);
        }
        return unauthorized();
      }),
    );

    render(<App />);
    expect(await screen.findByText("Choose a new password.")).toBeInTheDocument();

    await userEvent.type(screen.getByLabelText("New password"), "Reset-Password-123!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Reset-Password-123!");
    await userEvent.click(screen.getByRole("button", { name: /reset password/i }));

    await waitFor(() => expect(screen.getByText("Administration")).toBeInTheDocument());
  });

  it("blocks submission and reports mismatched passwords without calling the API", async () => {
    let called = false;
    gotoReset("reset-tok-1");
    server.use(
      http.get("/api/auth/me", () => unauthorized()),
      http.post("/api/auth/reset-password", () => {
        called = true;
        return unauthorized();
      }),
    );

    render(<App />);
    await screen.findByText("Choose a new password.");

    await userEvent.type(screen.getByLabelText("New password"), "Reset-Password-123!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Different-Pass-123!");
    await userEvent.click(screen.getByRole("button", { name: /reset password/i }));

    expect(await screen.findByText(/passwords don’t match/i)).toBeInTheDocument();
    expect(called).toBe(false);
  });

  it("surfaces an invalid / expired reset token", async () => {
    gotoReset("stale");
    server.use(
      http.get("/api/auth/me", () => unauthorized()),
      http.post("/api/auth/reset-password", () => unauthorized()),
    );

    render(<App />);
    await screen.findByText("Choose a new password.");

    await userEvent.type(screen.getByLabelText("New password"), "Reset-Password-123!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Reset-Password-123!");
    await userEvent.click(screen.getByRole("button", { name: /reset password/i }));

    expect(await screen.findByText(/invalid or has expired/i)).toBeInTheDocument();
  });
});
