// The charcoal sidebar and the page topbar, recreated from shell.jsx.

import React from "react";
import { I, type IconName } from "./icons";
import { Avatar } from "./ui";
import type { MeDto } from "../lib/types";

export type PageId = "users" | "roles" | "org" | "audit" | "keys";

const NAV: { id: PageId; label: string; icon: IconName }[] = [
  { id: "users", label: "Users", icon: "users" },
  { id: "roles", label: "Roles & Permissions", icon: "shield" },
  { id: "org", label: "Organization", icon: "building" },
  { id: "audit", label: "Audit Log", icon: "scroll" },
  { id: "keys", label: "API Keys", icon: "key" },
];

export function Sidebar({
  page,
  onNavigate,
  me,
  counts,
}: {
  page: PageId;
  onNavigate: (p: PageId) => void;
  me: MeDto;
  counts?: Partial<Record<PageId, number>>;
}) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <img src="/assets/madespace-icon.png" alt="" />
        <div className="wm">
          made space<small>Core · Admin</small>
        </div>
      </div>
      <nav className="nav">
        <div className="nav-label">Administration</div>
        {NAV.map((n) => {
          const Icon = I[n.icon];
          const count = counts?.[n.id];
          return (
            <button
              key={n.id}
              className={"nav-item" + (page === n.id ? " active" : "")}
              onClick={() => onNavigate(n.id)}
            >
              <Icon />
              <span>{n.label}</span>
              {count != null && <span className="count">{count}</span>}
            </button>
          );
        })}
      </nav>
      <div className="side-foot">
        <div className="who">
          <Avatar name={me.user.name} role={me.user.roleKey} size={36} />
          <div>
            <div className="nm">{me.user.name}</div>
            <div className="rl">
              {me.user.roleName} · {me.user.scope ?? "Studio"}
            </div>
          </div>
        </div>
      </div>
    </aside>
  );
}

export function Topbar({
  eyebrow,
  title,
  actions,
}: {
  eyebrow: string;
  title: string;
  actions?: React.ReactNode;
}) {
  return (
    <header className="topbar">
      <div className="tt">
        <div className="eyebrow-row">
          <span className="rule" />
          <span className="eb">{eyebrow}</span>
        </div>
        <h1>{title}</h1>
      </div>
      <div className="spacer" />
      {actions}
    </header>
  );
}
