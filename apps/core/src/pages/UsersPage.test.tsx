import { describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../test/server";
import { meOwner, org } from "../test/fixtures";
import type { RoleDto, UserDto } from "../lib/types";
import { AppProvider } from "../store/AppContext";
import { UsersPage } from "./UsersPage";

const roles: RoleDto[] = [
  { id: "r-owner", key: "owner", name: "Owner", isColumn: true, custom: false, protected: true, inherits: "Admin", rank: 60 },
  { id: "r-admin", key: "admin", name: "Admin", isColumn: true, custom: false, protected: false, inherits: "Manager", rank: 50 },
  { id: "r-manager", key: "manager", name: "Manager", isColumn: true, custom: false, protected: false, inherits: "Member", rank: 40 },
  { id: "r-member", key: "member", name: "Member", isColumn: true, custom: false, protected: false, inherits: "Viewer", rank: 30 },
  { id: "r-viewer", key: "viewer", name: "Viewer", isColumn: true, custom: false, protected: false, inherits: "—", rank: 20 },
];

interface CreateBody {
  name: string;
  email: string;
  role: string;
  scope?: string;
  password: string;
}

function handlers(onCreate: (b: CreateBody) => void) {
  return [
    http.get("/api/auth/me", () => HttpResponse.json(meOwner)),
    http.get("/api/org", () => HttpResponse.json(org)),
    http.get("/api/roles", () => HttpResponse.json(roles)),
    http.get("/api/users", () => HttpResponse.json([] as UserDto[])),
    http.post("/api/users", async ({ request }) => {
      const body = (await request.json()) as CreateBody;
      onCreate(body);
      const dto: UserDto = {
        id: "u-new",
        name: body.name,
        email: body.email,
        roleId: "r-manager",
        roleKey: body.role,
        roleName: "Manager",
        status: "active",
        scope: body.scope ?? null,
        lastActive: null,
        createdAt: "2026-05-30T00:00:00Z",
      };
      return HttpResponse.json(dto);
    }),
  ];
}

function renderPage() {
  return render(
    <AppProvider>
      <UsersPage />
    </AppProvider>,
  );
}

describe("UsersPage — manual create", () => {
  it("creates a user with basic info + initial password", async () => {
    const calls: CreateBody[] = [];
    server.use(...handlers((b) => calls.push(b)));
    renderPage();

    await userEvent.click(await screen.findByRole("button", { name: /new user/i }));
    expect(await screen.findByRole("heading", { name: "Create a user" })).toBeInTheDocument();

    await userEvent.type(screen.getByLabelText("Full name"), "Marcus Webb");
    await userEvent.type(screen.getByLabelText("Email address"), "marcus@madespace.co");
    await userEvent.type(screen.getByLabelText("Initial password"), "Sufficient1!");
    await userEvent.type(screen.getByLabelText("Confirm password"), "Sufficient1!");
    await userEvent.click(screen.getByRole("button", { name: "Create user" }));

    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({
      name: "Marcus Webb",
      email: "marcus@madespace.co",
      role: "member",
      password: "Sufficient1!",
    });
  });

  it("blocks a weak password client-side (no request)", async () => {
    const calls: CreateBody[] = [];
    server.use(...handlers((b) => calls.push(b)));
    renderPage();

    await userEvent.click(await screen.findByRole("button", { name: /new user/i }));
    await userEvent.type(screen.getByLabelText("Full name"), "Weak");
    await userEvent.type(screen.getByLabelText("Email address"), "weak@madespace.co");
    await userEvent.type(screen.getByLabelText("Initial password"), "short");
    await userEvent.type(screen.getByLabelText("Confirm password"), "short");
    await userEvent.click(screen.getByRole("button", { name: "Create user" }));

    expect(await screen.findByText(/at least 12 characters/i)).toBeInTheDocument();
    expect(calls).toHaveLength(0);
  });
});
