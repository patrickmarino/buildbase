// Roles & Permissions — the ACL matrix centerpiece. Click a cell to cycle its
// permission (allow → scope → deny); locked cells are rejected. Custom roles
// inherit from a base. Every change is audited server-side.

import React, { useCallback, useState } from "react";
import { Topbar } from "../components/Shell";
import { Modal } from "../components/Modal";
import { Eyebrow } from "../components/ui";
import { I } from "../components/icons";
import { api, ApiError } from "../lib/api";
import { useAsync } from "../lib/useAsync";
import { buildCellMap, cellKey, STATE_LABEL } from "../lib/matrix";
import type { CellDto, PermissionState, RoleDto } from "../lib/types";
import { useStore } from "../store/AppContext";

const SWATCH: Record<string, string> = {
  owner: "#8C7140",
  admin: "#B89A60",
  manager: "#4D4D4F",
  member: "#A7A9AC",
  viewer: "#E6E7E8",
  guest: "#9C8459",
  service: "#5E6B73",
};

function Mark({ state }: { state: PermissionState }) {
  if (state === "allow")
    return (
      <span className="mark allow">
        <I.check />
      </span>
    );
  if (state === "scope")
    return (
      <span className="mark scope">
        <I.dot />
      </span>
    );
  return (
    <span className="mark deny">
      <I.minus />
    </span>
  );
}

interface Tip {
  x: number;
  y: number;
  label: string;
  body: string;
}

export function RolesPage() {
  const { toast } = useStore();
  const load = useCallback(() => Promise.all([api.matrix(), api.listRoles()]), []);
  const { data, loading, setData, reload } = useAsync(load);
  const [sel, setSel] = useState<string | null>(null);
  const [tip, setTip] = useState<Tip | null>(null);
  const [newRole, setNewRole] = useState(false);

  if (loading || !data) return <Loading />;
  const [matrix, roles] = data;
  const cellMap = buildCellMap(matrix.cells);
  const roleKeyById = new Map(matrix.columns.map((r) => [r.id, r.key] as const));
  const actionLabel = new Map(matrix.groups.flatMap((g) => g.actions.map((a) => [a.key, a.label] as const)));

  const cycle = async (cell: CellDto) => {
    const roleKey = roleKeyById.get(cell.roleId) ?? "";
    if (cell.locked) {
      toast(roleKey === "owner" ? "Owner has full control by definition." : "This permission is locked by rule.", "lock");
      return;
    }
    try {
      const result = await api.cycleCell(cell.actionKey, cell.roleId);
      // optimistic local update with the server result
      setData((prev) => {
        if (!prev) return prev;
        const [m, rs] = prev;
        const cells = m.cells.map((c) =>
          c.actionKey === result.actionKey && c.roleId === result.roleId ? { ...c, state: result.state } : c,
        );
        return [{ ...m, cells }, rs];
      });
      toast(
        <span>
          <b>{actionLabel.get(cell.actionKey) ?? cell.actionKey}</b> for {roles.find((r) => r.id === cell.roleId)?.name} → {STATE_LABEL[result.state]}
        </span>,
      );
    } catch (e) {
      toast(e instanceof ApiError ? e.message : "Could not update permission.", "lock");
    }
  };

  const showTip = (e: React.MouseEvent, label: string, body: string) => {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    setTip({ x: r.left + r.width / 2, y: r.top - 10, label, body });
  };

  const cols = matrix.columns;
  const totalActions = matrix.groups.reduce((n, g) => n + g.actions.length, 0);

  return (
    <div className="main">
      <Topbar
        eyebrow="Access control"
        title="Roles & Permissions"
        actions={
          <button className="btn btn-gold" onClick={() => setNewRole(true)}>
            <I.plus />
            New role
          </button>
        }
      />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">
            The single source of truth for who can do what. Roles inherit downward — each includes everything below it.
            Click any cell to cycle its permission. Every change is written to the audit log.
          </p>

          <div className="grid-2">
            {/* Role rail */}
            <div>
              <Eyebrow label="The role set" />
              <div className="role-rail">
                {roles.map((r) => (
                  <div
                    key={r.id}
                    className={"role-card" + (sel === r.id ? " sel" : "")}
                    onClick={() => r.isColumn && setSel(sel === r.id ? null : r.id)}
                  >
                    <span
                      className="swatch"
                      style={{ background: SWATCH[r.key] ?? "#888", boxShadow: r.key === "viewer" ? "inset 0 0 0 1px var(--line-strong)" : "none" }}
                    />
                    <div>
                      <div className="rct">
                        {r.name}
                        {r.protected && <I.lock className="lockmini" />}
                        {r.custom && (
                          <span className="rchip" style={{ padding: "2px 7px", fontSize: "9px" }}>
                            Custom
                          </span>
                        )}
                      </div>
                      <div className="rci">{r.inherits === "—" ? "Base role" : "Inherits " + r.inherits}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Matrix */}
            <div className="panel">
              <div className="panel-head">
                <div>
                  <h3 className="ptitle">Permission matrix</h3>
                  <p className="psub">
                    {cols.length} roles · {totalActions} actions · deny-by-default
                  </p>
                </div>
                <div className="spacer" />
                <div className="legend">
                  <span className="li">
                    <Mark state="allow" />
                    Allowed
                  </span>
                  <span className="li">
                    <Mark state="scope" />
                    Scoped
                  </span>
                  <span className="li">
                    <Mark state="deny" />
                    Denied
                  </span>
                </div>
              </div>
              <div className="matrix-wrap">
                <table className="matrix">
                  <thead>
                    <tr>
                      <th className="corner" />
                      {cols.map((r) => (
                        <th
                          key={r.id}
                          className={"role-col" + (sel === r.id ? " sel" : "") + (r.protected ? " protected" : "")}
                          onClick={() => setSel(sel === r.id ? null : r.id)}
                          style={{ cursor: "pointer" }}
                        >
                          <div className="rc-name">
                            <span className="swatch" style={{ background: SWATCH[r.key], boxShadow: r.key === "viewer" ? "inset 0 0 0 1px var(--line-strong)" : "none" }} />
                            {r.name}
                          </div>
                          <div className="rc-inh">{r.inherits === "—" ? "base" : "↓ " + r.inherits}</div>
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {matrix.groups.map((g, gi) => (
                      <React.Fragment key={g.category}>
                        <tr className={"group-row" + (gi === 0 ? " first" : "")}>
                          <td colSpan={cols.length + 1}>
                            <div className="glabel">
                              <span className="rule" />
                              <span>{g.label}</span>
                            </div>
                          </td>
                        </tr>
                        {g.actions.map((a) => (
                          <tr key={a.key} className="act-row">
                            <td className="act">
                              <span className="alabel">{a.label}</span>
                            </td>
                            {cols.map((r) => {
                              const cell = cellMap.get(cellKey(a.key, r.id));
                              const state = cell?.state ?? "deny";
                              const locked = cell?.locked ?? false;
                              return (
                                <td key={r.id} className={"cell" + (sel === r.id ? " sel" : "")}>
                                  <button
                                    className={"cellbtn" + (locked ? " locked" : "")}
                                    onClick={() => cell && cycle(cell)}
                                    onMouseEnter={(e) =>
                                      showTip(
                                        e,
                                        STATE_LABEL[state] + (locked ? " · locked" : ""),
                                        locked ? (r.key === "owner" ? "Owner has full control by definition." : "Owner-only action.") : "Click to change",
                                      )
                                    }
                                    onMouseLeave={() => setTip(null)}
                                    aria-label={`${a.label} for ${r.name}: ${STATE_LABEL[state]}${locked ? " (locked)" : ""}`}
                                  >
                                    <Mark state={state} />
                                    {locked && <I.lock className="lockico" />}
                                  </button>
                                </td>
                              );
                            })}
                          </tr>
                        ))}
                      </React.Fragment>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      </div>

      {tip && (
        <div className="tip" style={{ left: tip.x, top: tip.y, transform: "translate(-50%,-100%)" }}>
          <span className="tt-lbl">{tip.label}</span>
          {tip.body}
        </div>
      )}

      {newRole && <NewRoleModal columns={cols} onClose={() => setNewRole(false)} onCreated={() => reload()} />}
    </div>
  );
}

function NewRoleModal({ columns, onClose, onCreated }: { columns: RoleDto[]; onClose: () => void; onCreated: () => void }) {
  const { toast } = useStore();
  const bases = columns.filter((r) => r.key !== "owner");
  const [name, setName] = useState("");
  const [base, setBase] = useState(bases.find((b) => b.key === "viewer")?.id ?? bases[0]?.id ?? "");
  const [err, setErr] = useState("");

  const create = async () => {
    const nm = name.trim();
    if (!nm) {
      setErr("Give the role a name.");
      return;
    }
    try {
      const role = await api.createRole(nm, base);
      toast(
        <span>
          Role <b>{role.name}</b> created — inherits {role.inherits}.
        </span>,
      );
      onCreated();
      onClose();
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : "Could not create role.");
    }
  };

  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label="New role" />
        <h2>Create a custom role</h2>
      </div>
      <div className="modal-body">
        <div className="field">
          <label>Role name</label>
          <input
            className={"inp" + (err && !name.trim() ? " bad" : "")}
            value={name}
            autoFocus
            onChange={(e) => {
              setName(e.target.value);
              setErr("");
            }}
            placeholder="e.g. Stager, Auditor, Finance"
          />
          {err ? <div className="hint err">{err}</div> : <div className="hint">Shown wherever roles are assigned.</div>}
        </div>
        <div className="field">
          <label>Inherits from</label>
          <select className="inp" value={base} onChange={(e) => setBase(e.target.value)}>
            {bases.map((r) => (
              <option key={r.id} value={r.id}>
                {r.name}
              </option>
            ))}
          </select>
          <div className="hint">
            The new role starts with everything the chosen role can do — then you tune it in the matrix. It can never
            exceed your own level.
          </div>
        </div>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button className="btn btn-gold" onClick={create}>
          <I.plus />
          Create role
        </button>
      </div>
    </Modal>
  );
}

function Loading() {
  return (
    <div className="main">
      <Topbar eyebrow="Access control" title="Roles & Permissions" />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">Loading…</p>
        </div>
      </div>
    </div>
  );
}
