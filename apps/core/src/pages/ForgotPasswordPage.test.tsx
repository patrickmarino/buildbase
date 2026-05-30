import { afterEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../test/server";
import { App } from "../App";

const unauthorized = () =>
  new HttpResponse(JSON.stringify({ error: "unauthorized", message: "no" }), {
    status: 401,
    headers: { "content-type": "application/json" },
  });

describe("ForgotPasswordPage", () => {
  afterEach(() => window.history.replaceState(null, "", "/"));

  it("requests a reset link and shows the neutral confirmation", async () => {
    window.history.replaceState(null, "", "/forgot-password");
    let requested: string | null = null;
    server.use(
      http.get("/api/auth/me", () => unauthorized()),
      http.post("/api/auth/forgot-password", async ({ request }) => {
        const body = (await request.json()) as { email: string };
        requested = body.email;
        return new HttpResponse(null, { status: 204 });
      }),
    );

    render(<App />);
    expect(await screen.findByText("Reset your password.")).toBeInTheDocument();

    await userEvent.type(screen.getByLabelText("Email address"), "elena@madespace.co");
    await userEvent.click(screen.getByRole("button", { name: /send reset link/i }));

    expect(await screen.findByText("Check your inbox.")).toBeInTheDocument();
    expect(requested).toBe("elena@madespace.co");
  });

  it("shows the same confirmation even if the request errors (no enumeration)", async () => {
    window.history.replaceState(null, "", "/forgot-password");
    server.use(
      http.get("/api/auth/me", () => unauthorized()),
      http.post("/api/auth/forgot-password", () =>
        new HttpResponse(JSON.stringify({ error: "server", message: "boom" }), {
          status: 500,
          headers: { "content-type": "application/json" },
        }),
      ),
    );

    render(<App />);
    await screen.findByText("Reset your password.");

    await userEvent.type(screen.getByLabelText("Email address"), "ghost@madespace.co");
    await userEvent.click(screen.getByRole("button", { name: /send reset link/i }));

    expect(await screen.findByText("Check your inbox.")).toBeInTheDocument();
  });
});
