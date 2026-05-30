import { describe, expect, it } from "vitest";
import { buildCellMap, cellKey, cycle, isLocked, lockReason, NEXT, STATE_LABEL } from "./matrix";
import type { CellDto } from "./types";

describe("matrix helpers", () => {
  it("cycles allow → scope → deny → allow", () => {
    expect(cycle("allow")).toBe("scope");
    expect(cycle("scope")).toBe("deny");
    expect(cycle("deny")).toBe("allow");
    // NEXT and cycle agree
    expect(NEXT.allow).toBe("scope");
  });

  it("labels each state", () => {
    expect(STATE_LABEL.allow).toBe("Allowed");
    expect(STATE_LABEL.scope).toBe("Scoped");
    expect(STATE_LABEL.deny).toBe("Denied");
  });

  it("locks the owner column and owner-only actions", () => {
    expect(isLocked("users.invite", "owner")).toBe(true);
    expect(isLocked("org.delete", "admin")).toBe(true);
    expect(isLocked("org.transfer", "manager")).toBe(true);
    expect(isLocked("roles.owner", "admin")).toBe(true);
    expect(isLocked("users.invite", "admin")).toBe(false);
  });

  it("gives a lock reason or null", () => {
    expect(lockReason("users.invite", "owner")).toMatch(/full control/i);
    expect(lockReason("roles.owner", "admin")).toMatch(/grant ownership/i);
    expect(lockReason("org.delete", "admin")).toMatch(/owner-only/i);
    expect(lockReason("users.invite", "admin")).toBeNull();
  });

  it("indexes cells by action+role", () => {
    const cells: CellDto[] = [
      { actionKey: "users.invite", roleId: "r1", state: "allow", locked: false },
      { actionKey: "users.invite", roleId: "r2", state: "deny", locked: false },
    ];
    const map = buildCellMap(cells);
    expect(map.get(cellKey("users.invite", "r1"))?.state).toBe("allow");
    expect(map.get(cellKey("users.invite", "r2"))?.state).toBe("deny");
    expect(map.get(cellKey("missing", "r1"))).toBeUndefined();
  });
});
