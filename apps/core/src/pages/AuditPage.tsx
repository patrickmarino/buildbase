// Audit Log — search, category filter, expandable before→after diffs, CSV export.

import React, { useCallback, useState } from "react";
import { Topbar } from "../components/Shell";
import { Avatar, RoleChip, Segmented } from "../components/ui";
import { I, type IconName } from "../components/icons";
import { api } from "../lib/api";
import { useAsync } from "../lib/useAsync";
import { formatTimestamp } from "../lib/format";
import { useStore } from "../store/AppContext";

const ACTION_ICON: Record<string, IconName> = {
  "role.assign": "shield",
  "role.create": "shield",
  "permission.edit": "shield",
  "org.settings.update": "building",
  "org.ownership.transfer": "refresh",
  "org.ownership.accept": "refresh",
  "org.delete": "building",
  "user.invite": "mail",
  "user.deactivate": "userX",
  "user.reactivate": "refresh",
  "user.edit": "edit",
  "key.create": "key",
  "key.revoke": "key",
  "audit.export": "download",
};
const CAT_LABEL: Record<string, string> = { users: "Users", roles: "Roles", org: "Org", audit: "Audit", keys: "Keys" };

export function AuditPage() {
  const { toast } = useStore();
  const load = useCallback(() => api.audit(), []);
  const { data, loading } = useAsync(load);
  const [q, setQ] = useState("");
  const [cat, setCat] = useState("all");
  const [open, setOpen] = useState<Record<string, boolean>>({});

  if (loading || !data) return <Loading />;

  const filtered = data.filter(
    (e) =>
      (cat === "all" || e.category === cat) &&
      (q === "" || `${e.actor.name} ${e.action} ${e.target ?? ""}`.toLowerCase().includes(q.toLowerCase())),
  );

  const exportCsv = async () => {
    const res = await fetch(api.auditExportUrl({ q, category: cat }), { credentials: "include" });
    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "audit-log.csv";
    a.click();
    URL.revokeObjectURL(url);
    toast(
      <span>
        Exported <b>{filtered.length}</b> events to CSV.
      </span>,
      "download",
    );
  };

  return (
    <div className="main">
      <Topbar
        eyebrow="Accountability"
        title="Audit Log"
        actions={
          <button className="btn btn-outline" onClick={() => void exportCsv()}>
            <I.download />
            Export CSV
          </button>
        }
      />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">
            An append-only, tamper-evident record of every sensitive action — actor, target, before and after,
            timestamp, and IP. Immutable and retained per policy.
          </p>

          <div className="toolbar">
            <div className="search">
              <I.search />
              <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search actor, action, or target…" />
            </div>
            <Segmented
              value={cat}
              onChange={setCat}
              options={[
                { value: "all", label: "All" },
                ...["users", "roles", "org", "keys", "audit"].map((c) => ({ value: c, label: CAT_LABEL[c] })),
              ]}
            />
            <div style={{ flex: 1 }} />
            <span className="pill">{filtered.length} events</span>
          </div>

          <div className="panel">
            <table className="tbl">
              <thead>
                <tr>
                  <th style={{ width: 40 }} />
                  <th>Timestamp</th>
                  <th>Actor</th>
                  <th>Action</th>
                  <th>Target</th>
                  <th style={{ width: 140 }}>IP</th>
                </tr>
              </thead>
              <tbody>
                {filtered.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="empty">
                      No events match.
                    </td>
                  </tr>
                ) : (
                  filtered.map((e) => {
                    const isOpen = open[e.id];
                    const ActionIcon = I[ACTION_ICON[e.action] ?? "dot"];
                    return (
                      <React.Fragment key={e.id}>
                        <tr style={{ cursor: "pointer" }} onClick={() => setOpen((o) => ({ ...o, [e.id]: !o[e.id] }))}>
                          <td>
                            <span style={{ display: "inline-flex", color: "var(--ink-subtle)", transition: "transform var(--dur-fast)", transform: isOpen ? "rotate(90deg)" : "none" }}>
                              <I.chevronRight style={{ width: 16, height: 16 }} />
                            </span>
                          </td>
                          <td>
                            <span className="mono">{formatTimestamp(e.ts)}</span>
                          </td>
                          <td>
                            <div className="uname">
                              <Avatar name={e.actor.name} role={e.actor.role} size={30} />
                              <div>
                                <div className="nm" style={{ fontSize: 13 }}>
                                  {e.actor.name}
                                </div>
                                <div style={{ marginTop: 2 }}>
                                  <RoleChip roleKey={e.actor.role} name={e.actor.role} />
                                </div>
                              </div>
                            </div>
                          </td>
                          <td>
                            <span style={{ display: "inline-flex", alignItems: "center", gap: 9 }}>
                              <span style={{ color: "var(--accent-strong)", display: "inline-flex" }}>
                                <ActionIcon style={{ width: 16, height: 16, strokeWidth: 1.5 }} />
                              </span>
                              <span className="mono" style={{ fontSize: 12.5, color: "var(--ink)" }}>
                                {e.action}
                              </span>
                            </span>
                          </td>
                          <td>{e.target}</td>
                          <td>
                            <span className="mono" style={{ fontSize: 11.5 }}>
                              {e.ip}
                            </span>
                          </td>
                        </tr>
                        {isOpen && (
                          <tr>
                            <td />
                            <td colSpan={5} style={{ paddingTop: 0, paddingBottom: 18 }}>
                              <div style={{ display: "flex", alignItems: "center", gap: 18, background: "var(--surface-mute)", borderRadius: "var(--r)", padding: "14px 18px", maxWidth: 560 }}>
                                <div className="kv">
                                  <span className="k">Before</span>
                                  <span className="v" style={{ fontFamily: "var(--font-mono)", fontSize: 13 }}>
                                    {e.before ?? "—"}
                                  </span>
                                </div>
                                <span style={{ color: "var(--accent-strong)", display: "inline-flex" }}>
                                  <I.arrowRight style={{ width: 18, height: 18 }} />
                                </span>
                                <div className="kv">
                                  <span className="k">After</span>
                                  <span className="v" style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--accent-deep)" }}>
                                    {e.after ?? "—"}
                                  </span>
                                </div>
                                <div style={{ flex: 1 }} />
                                <div className="kv">
                                  <span className="k">Category</span>
                                  <span className="v">
                                    <span className="pill">{CAT_LABEL[e.category] ?? e.category}</span>
                                  </span>
                                </div>
                              </div>
                            </td>
                          </tr>
                        )}
                      </React.Fragment>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}

function Loading() {
  return (
    <div className="main">
      <Topbar eyebrow="Accountability" title="Audit Log" />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">Loading…</p>
        </div>
      </div>
    </div>
  );
}
