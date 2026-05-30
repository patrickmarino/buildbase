// Small shared presentational components, recreated from the prototype's
// shell.jsx / organization.jsx using the same CSS classes.

import React from "react";
import { I, type IconName } from "./icons";
import { initials } from "../lib/format";

const AVA_BG: Record<string, string> = {
  owner: "#8C7140",
  admin: "#B89A60",
  manager: "#4D4D4F",
  member: "#6E6E70",
  viewer: "#A7A9AC",
  guest: "#9C8459",
  service: "#5E6B73",
};

export function Avatar({ name, role, size = 34 }: { name: string; role: string; size?: number }) {
  return (
    <div
      className="ava"
      style={{ width: size, height: size, background: AVA_BG[role] ?? "#6E6E70", fontSize: size * 0.36 }}
    >
      {initials(name)}
    </div>
  );
}

export function RoleChip({ roleKey, name }: { roleKey: string; name: string }) {
  return (
    <span className="rchip" data-role={roleKey}>
      <span className="swatch" />
      {name}
    </span>
  );
}

const STATUS_LABEL: Record<string, string> = {
  active: "Active",
  invited: "Invited",
  deactivated: "Deactivated",
  revoked: "Revoked",
};

export function StatusPill({ status }: { status: string }) {
  return (
    <span className="pill" data-s={status}>
      <span className="dot" />
      {STATUS_LABEL[status] ?? status}
    </span>
  );
}

export function Toggle({ on, onChange }: { on: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="sw">
      <input type="checkbox" checked={on} onChange={(e) => onChange(e.target.checked)} />
      <span className="track" />
      <span className="knob" />
    </label>
  );
}

export interface SegOption {
  value: string;
  label: string;
}

export function Segmented({
  value,
  options,
  onChange,
}: {
  value: string;
  options: SegOption[];
  onChange: (v: string) => void;
}) {
  return (
    <div className="seg">
      {options.map((o) => (
        <button key={o.value} className={value === o.value ? "on" : ""} onClick={() => onChange(o.value)}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function SettingRow({
  icon,
  title,
  desc,
  control,
}: {
  icon?: IconName;
  title: string;
  desc?: string;
  control: React.ReactNode;
}) {
  const IconCmp = icon ? I[icon] : null;
  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 16,
        padding: "var(--space-5) var(--space-8)",
        borderBottom: "1px solid var(--line)",
      }}
    >
      {IconCmp && (
        <div
          style={{
            width: 34,
            height: 34,
            borderRadius: "var(--r)",
            background: "var(--surface-mute)",
            display: "grid",
            placeItems: "center",
            color: "var(--accent-strong)",
            flex: "none",
          }}
        >
          <IconCmp style={{ width: 18, height: 18, strokeWidth: 1.5 }} />
        </div>
      )}
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontFamily: "var(--font-heading)",
            fontSize: 13,
            letterSpacing: "0.06em",
            textTransform: "uppercase",
            color: "var(--ink)",
          }}
        >
          {title}
        </div>
        {desc && (
          <div
            style={{
              fontFamily: "var(--font-body)",
              fontSize: 13,
              color: "var(--ink-muted)",
              lineHeight: 1.55,
              marginTop: 4,
              maxWidth: "52ch",
            }}
          >
            {desc}
          </div>
        )}
      </div>
      <div style={{ flex: "none", display: "flex", alignItems: "center" }}>{control}</div>
    </div>
  );
}

export function Eyebrow({ label, danger }: { label: string; danger?: boolean }) {
  return (
    <div className="sec-eyebrow">
      <span className="rule" style={danger ? { background: "#B6442F" } : undefined} />
      <span style={danger ? { color: "#B6442F" } : undefined}>{label}</span>
    </div>
  );
}
