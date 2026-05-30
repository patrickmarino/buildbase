import { describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../test/server";
import { matrix, meOwner, org, roles } from "../test/fixtures";
import { cycle } from "../lib/matrix";
import { AppProvider } from "../store/AppContext";
import { RolesPage } from "./RolesPage";

interface CellReq {
  actionKey: string;
  roleId: string;
}

function baseHandlers(onPatch: (body: CellReq) => void) {
  return [
    http.get("/api/auth/me", () => HttpResponse.json(meOwner)),
    http.get("/api/org", () => HttpResponse.json(org)),
    http.get("/api/permissions/matrix", () => HttpResponse.json(matrix)),
    http.get("/api/roles", () => HttpResponse.json(roles)),
    http.patch("/api/permissions/matrix/cell", async ({ request }) => {
      const body = (await request.json()) as CellReq;
      onPatch(body);
      const cell = matrix.cells.find((c) => c.actionKey === body.actionKey && c.roleId === body.roleId)!;
      return HttpResponse.json({ actionKey: body.actionKey, roleId: body.roleId, state: cycle(cell.state) });
    }),
  ];
}

function renderPage() {
  return render(
    <AppProvider>
      <RolesPage />
    </AppProvider>,
  );
}

describe("RolesPage", () => {
  it("renders the matrix with role columns", async () => {
    server.use(...baseHandlers(() => {}));
    renderPage();
    expect(await screen.findByText("Permission matrix")).toBeInTheDocument();
    expect(screen.getByText("Invite user")).toBeInTheDocument();
    // owner / admin / viewer columns
    expect(screen.getAllByText("Owner").length).toBeGreaterThan(0);
  });

  it("cycles an editable cell via the API and reflects the new state", async () => {
    const calls: CellReq[] = [];
    server.use(...baseHandlers((b) => calls.push(b)));
    renderPage();

    const cell = await screen.findByLabelText("Invite user for Admin: Allowed");
    await userEvent.click(cell);

    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toEqual({ actionKey: "users.invite", roleId: "r-admin" });
    // optimistic UI now shows the cycled state
    expect(await screen.findByLabelText("Invite user for Admin: Scoped")).toBeInTheDocument();
  });

  it("does not call the API for a locked cell", async () => {
    const calls: CellReq[] = [];
    server.use(...baseHandlers((b) => calls.push(b)));
    renderPage();

    const locked = await screen.findByLabelText("Delete organization for Admin: Denied (locked)");
    await userEvent.click(locked);

    // a toast appears and nothing was sent
    expect(await screen.findByText(/owner-only|locked by rule/i)).toBeInTheDocument();
    expect(calls).toHaveLength(0);
  });
});
