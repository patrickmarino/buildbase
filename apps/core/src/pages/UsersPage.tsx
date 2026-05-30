// Users — list, filter, invite, change role, deactivate/reactivate.

import React, { useCallback, useEffect, useRef, useState } from "react";
import { Topbar } from "../components/Shell";
import { Modal, Confirm } from "../components/Modal";
import { Avatar, RoleChip, StatusPill, Segmented, Eyebrow } from "../components/ui";
import { I } from "../components/icons";
import { api, ApiError } from "../lib/api";
import { useAsync } from "../lib/useAsync";
import { relativeTime } from "../lib/format";
import type { RoleDto, UserDto } from "../lib/types";
import { useStore } from "../store/AppContext";

export function UsersPage() {
  const { toast } = useStore();
  const load = useCallback(() => Promise.all([api.listUsers(), api.listRoles()]), []);
  const { data, loading, reload } = useAsync(load);
  const [q, setQ] = useState("");
  const [roleF, setRoleF] = useState("all");
  const [statusF, setStatusF] = useState("all");
  const [invite, setInvite] = useState(false);
  const [createUser, setCreateUser] = useState(false);
  const [menu, setMenu] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<UserDto | null>(null);
  const [changeRole, setChangeRole] = useState<UserDto | null>(null);

  if (loading || !data) return <Empty title="Users" />;
  const [users, roles] = data;

  const filtered = users.filter(
    (u) =>
      (roleF === "all" || u.roleKey === roleF) &&
      (statusF === "all" || u.status === statusF) &&
      (q === "" || u.name.toLowerCase().includes(q.toLowerCase()) || u.email.toLowerCase().includes(q.toLowerCase())),
  );

  const setStatus = async (u: UserDto, status: "active" | "deactivated") => {
    try {
      await api.setStatus(u.id, status);
      await reload();
      toast(
        status === "deactivated" ? (
          <span>
            <b>{u.name}</b> deactivated — sessions and tokens revoked.
          </span>
        ) : (
          <span>
            <b>{u.name}</b> reactivated.
          </span>
        ),
        status === "deactivated" ? "userX" : "refresh",
      );
    } catch (e) {
      toast(e instanceof ApiError ? e.message : "Action failed.", "alert");
    }
  };

  const roleCounts: Record<string, number> = {};
  roles.forEach((r) => (roleCounts[r.key] = users.filter((u) => u.roleKey === r.key).length));

  return (
    <div className="main">
      <Topbar
        eyebrow="Identity"
        title="Users"
        actions={
          <>
            <button className="btn btn-outline" onClick={() => setInvite(true)}>
              <I.mail />
              Invite
            </button>
            <button className="btn btn-gold" onClick={() => setCreateUser(true)}>
              <I.plus />
              New user
            </button>
          </>
        }
      />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">
            Everyone with access to the studio. Invite people, assign roles, and deactivate accounts — deactivation
            revokes all sessions and tokens immediately.
          </p>

          <div className="toolbar">
            <div className="search">
              <I.search />
              <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search name or email…" />
            </div>
            <select className="inp" style={{ width: "auto", minWidth: 150 }} value={roleF} onChange={(e) => setRoleF(e.target.value)}>
              <option value="all">All roles</option>
              {roles
                .filter((r) => roleCounts[r.key] > 0)
                .map((r) => (
                  <option key={r.id} value={r.key}>
                    {r.name} ({roleCounts[r.key]})
                  </option>
                ))}
            </select>
            <Segmented
              value={statusF}
              onChange={setStatusF}
              options={[
                { value: "all", label: "All" },
                { value: "active", label: "Active" },
                { value: "invited", label: "Invited" },
                { value: "deactivated", label: "Deactivated" },
              ]}
            />
          </div>

          <div className="panel">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Role</th>
                  <th>Scope</th>
                  <th>Status</th>
                  <th>Last active</th>
                  <th style={{ textAlign: "right" }} />
                </tr>
              </thead>
              <tbody>
                {filtered.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="empty">
                      No one matches those filters.
                    </td>
                  </tr>
                ) : (
                  filtered.map((u) => (
                    <tr key={u.id} style={u.status === "deactivated" ? { opacity: 0.55 } : undefined}>
                      <td>
                        <div className="uname">
                          <Avatar name={u.name} role={u.roleKey} size={36} />
                          <div>
                            <div className="nm">{u.name}</div>
                            <div className="em">{u.email}</div>
                          </div>
                        </div>
                      </td>
                      <td>
                        <RoleChip roleKey={u.roleKey} name={u.roleName} />
                      </td>
                      <td>
                        <span className="mono">{u.scope ?? "—"}</span>
                      </td>
                      <td>
                        <StatusPill status={u.status} />
                      </td>
                      <td>
                        <span className="mono">{relativeTime(u.lastActive)}</span>
                      </td>
                      <td style={{ textAlign: "right", position: "relative" }}>
                        <button
                          className="btn-icon row-act"
                          style={{ marginLeft: "auto", opacity: menu === u.id ? 1 : undefined }}
                          onClick={() => setMenu(menu === u.id ? null : u.id)}
                          aria-label={`Actions for ${u.name}`}
                        >
                          <I.more />
                        </button>
                        {menu === u.id && (
                          <RowMenu
                            user={u}
                            onClose={() => setMenu(null)}
                            onRole={() => {
                              setChangeRole(u);
                              setMenu(null);
                            }}
                            onDeact={() => {
                              setConfirm(u);
                              setMenu(null);
                            }}
                            onReact={() => {
                              void setStatus(u, "active");
                              setMenu(null);
                            }}
                          />
                        )}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {invite && <InviteModal roles={roles} onClose={() => setInvite(false)} onDone={() => reload()} />}
      {createUser && <CreateUserModal roles={roles} onClose={() => setCreateUser(false)} onDone={() => reload()} />}
      {changeRole && (
        <ChangeRoleModal user={changeRole} roles={roles} onClose={() => setChangeRole(null)} onDone={() => reload()} />
      )}
      {confirm && (
        <Confirm
          title={`Deactivate ${confirm.name}?`}
          danger
          confirmLabel="Deactivate"
          body="All access is revoked immediately — active sessions and API tokens are invalidated, and owned resources are flagged for reassignment. This can be undone by reactivating the account."
          onConfirm={() => void setStatus(confirm, "deactivated")}
          onClose={() => setConfirm(null)}
        />
      )}
    </div>
  );
}

function RowMenu({
  user,
  onClose,
  onRole,
  onDeact,
  onReact,
}: {
  user: UserDto;
  onClose: () => void;
  onRole: () => void;
  onDeact: () => void;
  onReact: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const t = window.setTimeout(() => document.addEventListener("mousedown", onDoc), 0);
    return () => {
      window.clearTimeout(t);
      document.removeEventListener("mousedown", onDoc);
    };
  }, [onClose]);

  const item = (Icon: React.ComponentType<React.SVGProps<SVGSVGElement>>, label: string, fn: () => void, danger?: boolean) => (
    <button
      onClick={fn}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 11,
        width: "100%",
        padding: "10px 14px",
        border: "none",
        background: "none",
        cursor: "pointer",
        fontFamily: "var(--font-body)",
        fontSize: 13.5,
        color: danger ? "#B6442F" : "var(--ink)",
        textAlign: "left",
      }}
    >
      <Icon style={{ width: 16, height: 16, strokeWidth: 1.5 }} />
      {label}
    </button>
  );

  return (
    <div
      ref={ref}
      style={{
        position: "absolute",
        top: "calc(100% - 6px)",
        right: 12,
        zIndex: 30,
        minWidth: 186,
        background: "var(--surface)",
        border: "1px solid var(--line)",
        borderRadius: "var(--r)",
        boxShadow: "var(--shadow-pop)",
        padding: "6px 0",
        textAlign: "left",
      }}
    >
      {item(I.shield, "Change role", onRole)}
      <hr className="divider" style={{ margin: "6px 0" }} />
      {user.status === "deactivated" ? item(I.refresh, "Reactivate", onReact) : item(I.userX, "Deactivate", onDeact, true)}
    </div>
  );
}

function InviteModal({ roles, onClose, onDone }: { roles: RoleDto[]; onClose: () => void; onDone: () => void }) {
  const { toast } = useStore();
  const assignable = roles.filter((r) => ["admin", "manager", "member", "viewer"].includes(r.key));
  const [email, setEmail] = useState("");
  const [role, setRole] = useState("member");
  const [scope, setScope] = useState("Hampstead");
  const [err, setErr] = useState("");

  const send = async () => {
    const e = email.trim();
    if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e)) {
      setErr("Enter a valid email address.");
      return;
    }
    try {
      await api.invite(e, role, scope);
      toast(
        <span>
          Invitation sent to <b>{e}</b> as {roles.find((r) => r.key === role)?.name}.
        </span>,
        "mail",
      );
      onDone();
      onClose();
    } catch (ex) {
      setErr(ex instanceof ApiError ? ex.message : "Could not send invitation.");
    }
  };

  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label="Invite" />
        <h2>Invite someone to the studio</h2>
      </div>
      <div className="modal-body">
        <div className="field">
          <label>Email address</label>
          <input
            className={"inp" + (err ? " bad" : "")}
            value={email}
            autoFocus
            type="email"
            onChange={(e) => {
              setEmail(e.target.value);
              setErr("");
            }}
            placeholder="name@madespace.co"
          />
          {err ? <div className="hint err">{err}</div> : <div className="hint">They’ll receive a link to set a password and enrol in MFA.</div>}
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--space-5)" }}>
          <div className="field">
            <label>Role</label>
            <select className="inp" value={role} onChange={(e) => setRole(e.target.value)}>
              {assignable.map((r) => (
                <option key={r.id} value={r.key}>
                  {r.name}
                </option>
              ))}
            </select>
            <div className="hint">Default is Member.</div>
          </div>
          <div className="field">
            <label>Scope</label>
            <select className="inp" value={scope} onChange={(e) => setScope(e.target.value)}>
              {["Studio", "Hampstead", "Notting Hill", "Auditor"].map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button className="btn btn-gold" onClick={send}>
          <I.mail />
          Send invitation
        </button>
      </div>
    </Modal>
  );
}

function CreateUserModal({ roles, onClose, onDone }: { roles: RoleDto[]; onClose: () => void; onDone: () => void }) {
  const { toast } = useStore();
  const assignable = roles.filter((r) => ["admin", "manager", "member", "viewer"].includes(r.key));
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [role, setRole] = useState("member");
  const [scope, setScope] = useState("Studio");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [err, setErr] = useState("");

  const policyOk = password.length >= 12 && /\d/.test(password) && /[^A-Za-z0-9\s]/.test(password);

  const create = async () => {
    if (!name.trim()) return setErr("Enter the person’s name.");
    if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email.trim())) return setErr("Enter a valid email address.");
    if (!policyOk) return setErr("Password must be at least 12 characters and include a number and a symbol.");
    if (password !== confirm) return setErr("Passwords don’t match.");
    try {
      const u = await api.createUser({ name: name.trim(), email: email.trim(), role, scope, password });
      toast(
        <span>
          <b>{u.name}</b> created as {roles.find((r) => r.key === role)?.name} — they can sign in now.
        </span>,
        "users",
      );
      onDone();
      onClose();
    } catch (ex) {
      setErr(ex instanceof ApiError ? ex.message : "Could not create the user.");
    }
  };

  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label="New user" />
        <h2>Create a user</h2>
      </div>
      <div className="modal-body">
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--space-5)" }}>
          <div className="field">
            <label htmlFor="cu-name">Full name</label>
            <input
              id="cu-name"
              className={"inp" + (err && !name.trim() ? " bad" : "")}
              value={name}
              autoFocus
              onChange={(e) => {
                setName(e.target.value);
                setErr("");
              }}
              placeholder="e.g. Marcus Webb"
            />
          </div>
          <div className="field">
            <label htmlFor="cu-email">Email address</label>
            <input
              id="cu-email"
              className="inp"
              type="email"
              value={email}
              onChange={(e) => {
                setEmail(e.target.value);
                setErr("");
              }}
              placeholder="name@madespace.co"
            />
          </div>
          <div className="field">
            <label htmlFor="cu-role">Role</label>
            <select id="cu-role" className="inp" value={role} onChange={(e) => setRole(e.target.value)}>
              {assignable.map((r) => (
                <option key={r.id} value={r.key}>
                  {r.name}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label htmlFor="cu-scope">Scope</label>
            <select id="cu-scope" className="inp" value={scope} onChange={(e) => setScope(e.target.value)}>
              {["Studio", "Hampstead", "Notting Hill", "Auditor"].map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label htmlFor="cu-password">Initial password</label>
            <input
              id="cu-password"
              className="inp"
              type="password"
              value={password}
              onChange={(e) => {
                setPassword(e.target.value);
                setErr("");
              }}
              placeholder="••••••••••••"
            />
          </div>
          <div className="field">
            <label htmlFor="cu-confirm">Confirm password</label>
            <input
              id="cu-confirm"
              className="inp"
              type="password"
              value={confirm}
              onChange={(e) => {
                setConfirm(e.target.value);
                setErr("");
              }}
              placeholder="••••••••••••"
            />
          </div>
        </div>
        {err ? (
          <div className="hint err">{err}</div>
        ) : (
          <div className="hint">
            The account is active immediately. Password must be ≥ 12 characters with a number and a symbol; the user can
            change it after signing in.
          </div>
        )}
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button className="btn btn-gold" onClick={create}>
          <I.plus />
          Create user
        </button>
      </div>
    </Modal>
  );
}

function ChangeRoleModal({
  user,
  roles,
  onClose,
  onDone,
}: {
  user: UserDto;
  roles: RoleDto[];
  onClose: () => void;
  onDone: () => void;
}) {
  const { toast } = useStore();
  const opts = roles.filter((r) => r.isColumn || r.key === "guest");
  const [roleId, setRoleId] = useState(user.roleId);
  const [err, setErr] = useState("");
  const selected = roles.find((r) => r.id === roleId);

  const save = async () => {
    if (roleId === user.roleId) {
      onClose();
      return;
    }
    try {
      await api.changeRole(user.id, roleId);
      toast(
        <span>
          <b>{user.name}</b> is now {selected?.name}.
        </span>,
        "shield",
      );
      onDone();
      onClose();
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : "Could not change role.");
    }
  };

  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label="Assign role" />
        <h2>Change role · {user.name}</h2>
      </div>
      <div className="modal-body">
        <div className="field">
          <label>Role</label>
          <select className="inp" value={roleId} onChange={(e) => setRoleId(e.target.value)}>
            {opts.map((r) => (
              <option key={r.id} value={r.id}>
                {r.name}
              </option>
            ))}
          </select>
          {err ? (
            <div className="hint err">{err}</div>
          ) : selected?.key === "owner" ? (
            <div className="hint err">
              Granting Owner transfers ultimate control. The change takes effect immediately.
            </div>
          ) : (
            <div className="hint">A user can never be assigned a role higher than your own. The change takes effect immediately.</div>
          )}
        </div>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button className="btn btn-gold" onClick={save}>
          Save role
        </button>
      </div>
    </Modal>
  );
}

function Empty({ title }: { title: string }) {
  return (
    <div className="main">
      <Topbar eyebrow="Identity" title={title} />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">Loading…</p>
        </div>
      </div>
    </div>
  );
}
