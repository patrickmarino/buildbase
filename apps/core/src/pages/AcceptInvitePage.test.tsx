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

function gotoAcceptInvite(token: string) {
  window.history.replaceState(null, "", `/accept-invite?token=${token}`);
}

describe("AcceptInvitePage", () => {
  afterEach(() => window.history.replaceState(null, "", "/"));

  it("sets a password via the token and lands in the app shell", async () => {
    let activated = false;
    gotoAcceptInvite("invite-tok-1");
    server.use(
      http.get("/api/auth/me", () => (activated ? HttpResponse.json(meOwner) : unauthorized())),
      http.get("/api/org", () => HttpResponse.json(org)),
      http.get("/api/permissions/matrix", () => HttpResponse.json({ groups: [], columns: [], cells: [] })),
      http.get("/api/roles", () => HttpResponse.json([])),
      http.post("/api/auth/accept-invite", async ({ request }) => {
        const body = (await request.json()) as { token: string; password: string };
        if (body.token === "invite-tok-1" && body.password.length >= 8) {
          activated = true;
          return HttpResponse.json(meOwner);
        }
        return unauthorized();
      }),
    );

    render(<App />);

    expect(await screen.findByText("Set your password.")).toBeInTheDocument();

    await userEvent.type(screen.getByLabelText("New password"), "Sufficient1!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Sufficient1!");
    await userEvent.click(screen.getByRole("button", { name: /set password/i }));

    await waitFor(() => expect(screen.getByText("Administration")).toBeInTheDocument());
  });

  it("blocks submission and reports mismatched passwords without calling the API", async () => {
    let called = false;
    gotoAcceptInvite("invite-tok-1");
    server.use(
      http.get("/api/auth/me", () => unauthorized()),
      http.post("/api/auth/accept-invite", () => {
        called = true;
        return unauthorized();
      }),
    );

    render(<App />);
    await screen.findByText("Set your password.");

    await userEvent.type(screen.getByLabelText("New password"), "Sufficient1!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Different1!");
    await userEvent.click(screen.getByRole("button", { name: /set password/i }));

    expect(await screen.findByText(/passwords don’t match/i)).toBeInTheDocument();
    expect(called).toBe(false);
  });

  it("surfaces an invalid / expired invitation token", async () => {
    gotoAcceptInvite("stale");
    server.use(
      http.get("/api/auth/me", () => unauthorized()),
      http.post("/api/auth/accept-invite", () => unauthorized()),
    );

    render(<App />);
    await screen.findByText("Set your password.");

    await userEvent.type(screen.getByLabelText("New password"), "Sufficient1!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Sufficient1!");
    await userEvent.click(screen.getByRole("button", { name: /set password/i }));

    expect(await screen.findByText(/invalid or has expired/i)).toBeInTheDocument();
  });
});
