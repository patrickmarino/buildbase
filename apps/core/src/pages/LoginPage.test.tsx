import { describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../test/server";
import { meOwner, org } from "../test/fixtures";
import { App } from "../App";

describe("LoginPage / auth gate", () => {
  it("shows login when unauthenticated, then the app after a successful sign-in", async () => {
    let signedIn = false;
    server.use(
      http.get("/api/auth/me", () => (signedIn ? HttpResponse.json(meOwner) : new HttpResponse(JSON.stringify({ error: "unauthorized", message: "no" }), { status: 401, headers: { "content-type": "application/json" } }))),
      http.get("/api/org", () => HttpResponse.json(org)),
      http.get("/api/permissions/matrix", () => HttpResponse.json({ groups: [], columns: [], cells: [] })),
      http.get("/api/roles", () => HttpResponse.json([])),
      http.post("/api/auth/login", async ({ request }) => {
        const body = (await request.json()) as { email: string; password: string };
        if (body.password === "correct") {
          signedIn = true;
          return HttpResponse.json(meOwner);
        }
        return new HttpResponse(JSON.stringify({ error: "unauthorized", message: "bad" }), { status: 401, headers: { "content-type": "application/json" } });
      }),
    );

    render(<App />);

    // initially the sign-in screen
    expect(await screen.findByText("Welcome back.")).toBeInTheDocument();

    await userEvent.type(screen.getByLabelText("Email address"), "elena@madespace.co");
    await userEvent.type(screen.getByLabelText("Password"), "wrong");
    await userEvent.click(screen.getByRole("button", { name: /sign in/i }));
    expect(await screen.findByText(/don’t match/i)).toBeInTheDocument();

    // correct password → app shell (sidebar shows the signed-in owner)
    await userEvent.clear(screen.getByLabelText("Password"));
    await userEvent.type(screen.getByLabelText("Password"), "correct");
    await userEvent.click(screen.getByRole("button", { name: /sign in/i }));

    await waitFor(() => expect(screen.getByText("Administration")).toBeInTheDocument());
  });
});
