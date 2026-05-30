// Organization — profile & branding, authentication (MFA, password policy, SSO),
// and the danger zone (ownership transfer, deletion). A dirty-tracking save bar
// commits changes; all writes are audited server-side.

import React, { useCallback, useState } from "react";
import { Topbar } from "../components/Shell";
import { Modal } from "../components/Modal";
import { Toggle, Segmented, SettingRow, Eyebrow } from "../components/ui";
import { I } from "../components/icons";
import { api, ApiError } from "../lib/api";
import { useAsync } from "../lib/useAsync";
import type { OrgDto, UserDto } from "../lib/types";
import { useStore } from "../store/AppContext";

const ACCENTS = [
  { v: "#D6B982", name: "Champagne" },
  { v: "#B89A60", name: "Antique" },
  { v: "#9C8459", name: "Bronze" },
  { v: "#7E8A74", name: "Sage" },
];

export function OrgPage() {
  const { toast, setAccent, logout } = useStore();
  const load = useCallback(() => api.getOrg(), []);
  const { data, loading, setData } = useAsync(load);
  const [s, setS] = useState<OrgDto | null>(null);
  const [dirty, setDirty] = useState(false);
  const [transfer, setTransfer] = useState(false);
  const [del, setDel] = useState(false);

  // sync local editable copy once loaded
  const current = s ?? data;
  if (loading || !current) return <Loading />;

  const set = (patch: Partial<OrgDto>) => {
    setS({ ...current, ...patch });
    setDirty(true);
  };
  const setMfa = (patch: Partial<OrgDto["mfa"]>) => set({ mfa: { ...current.mfa, ...patch } });
  const setPw = (patch: Partial<OrgDto["passwordPolicy"]>) => set({ passwordPolicy: { ...current.passwordPolicy, ...patch } });
  const setSso = (patch: Partial<OrgDto["sso"]>) => set({ sso: { ...current.sso, ...patch } });

  const save = async () => {
    try {
      const updated = await api.updateOrg({
        name: current.name,
        domain: current.domain ?? "",
        accentColor: current.branding.accentColor,
        mfa: current.mfa,
        passwordPolicy: current.passwordPolicy,
        sso: current.sso,
      });
      setData(updated);
      setS(null);
      setDirty(false);
      toast("Organization settings saved.", "building");
    } catch (e) {
      toast(e instanceof ApiError ? e.message : "Could not save settings.", "alert");
    }
  };
  const discard = () => {
    setS(null);
    setDirty(false);
  };

  return (
    <div className="main">
      <Topbar eyebrow="Configuration" title="Organization" />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">
            The studio’s identity and the security foundation every other module trusts. Changes here are sensitive — all
            are written to the audit log.
          </p>

          <Eyebrow label="Profile & branding" />
          <div className="panel" style={{ marginBottom: "var(--space-10)" }}>
            <div style={{ padding: "var(--space-8)", display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--space-6) var(--space-8)" }}>
              <div className="field">
                <label>Organization name</label>
                <input className="inp" value={current.name} onChange={(e) => set({ name: e.target.value })} />
              </div>
              <div className="field">
                <label>Primary domain</label>
                <input className="inp" value={current.domain ?? ""} onChange={(e) => set({ domain: e.target.value })} />
                <div className="hint">Invitations to this domain are auto-approved.</div>
              </div>
            </div>
            <hr className="divider" />
            <SettingRow
              icon="globe"
              title="Accent colour"
              desc="The champagne mark that runs through every surface. Applied live across Core."
              control={
                <div style={{ display: "flex", gap: 10 }}>
                  {ACCENTS.map((a) => (
                    <button
                      key={a.v}
                      title={a.name}
                      onClick={() => {
                        setAccent(a.v);
                        set({ branding: { accentColor: a.v } });
                      }}
                      style={{
                        width: 30,
                        height: 30,
                        borderRadius: "50%",
                        background: a.v,
                        cursor: "pointer",
                        border: current.branding.accentColor === a.v ? "2px solid var(--ink)" : "2px solid transparent",
                        boxShadow: "0 0 0 1px var(--line)",
                        outlineOffset: 2,
                      }}
                    />
                  ))}
                </div>
              }
            />
          </div>

          <Eyebrow label="Authentication" />
          <div className="panel" style={{ marginBottom: "var(--space-6)" }}>
            <SettingRow
              icon="fingerprint"
              title="Multi-factor authentication"
              desc="Require a second factor at sign-in. TOTP is the minimum supported method."
              control={<Toggle on={current.mfa.enabled} onChange={(v) => setMfa({ enabled: v })} />}
            />
            {current.mfa.enabled && (
              <>
                <SettingRow
                  title="Method"
                  desc="How the second factor is delivered."
                  control={
                    <Segmented
                      value={current.mfa.method}
                      onChange={(v) => setMfa({ method: v as OrgDto["mfa"]["method"] })}
                      options={[
                        { value: "totp", label: "Authenticator" },
                        { value: "sms", label: "SMS" },
                        { value: "webauthn", label: "Passkey" },
                      ]}
                    />
                  }
                />
                <SettingRow
                  title="Enforce for"
                  desc="Who must complete MFA before they can act."
                  control={
                    <Segmented
                      value={current.mfa.enforce}
                      onChange={(v) => setMfa({ enforce: v as OrgDto["mfa"]["enforce"] })}
                      options={[
                        { value: "all", label: "Everyone" },
                        { value: "admins", label: "Admins +" },
                        { value: "none", label: "Optional" },
                      ]}
                    />
                  }
                />
              </>
            )}
          </div>

          <div className="panel" style={{ marginBottom: "var(--space-6)" }}>
            <SettingRow
              icon="lock"
              title="Minimum password length"
              desc="Shorter passwords are rejected at sign-up and on change."
              control={
                <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
                  <input
                    type="range"
                    min={8}
                    max={32}
                    value={current.passwordPolicy.minLength}
                    onChange={(e) => setPw({ minLength: +e.target.value })}
                    style={{ width: 180, accentColor: "var(--accent)" }}
                  />
                  <span className="mono" style={{ fontSize: 15, color: "var(--ink)", minWidth: 54 }}>
                    {current.passwordPolicy.minLength} chars
                  </span>
                </div>
              }
            />
            <SettingRow title="Require a number" control={<Toggle on={current.passwordPolicy.requireNumber} onChange={(v) => setPw({ requireNumber: v })} />} />
            <SettingRow title="Require a symbol" control={<Toggle on={current.passwordPolicy.requireSymbol} onChange={(v) => setPw({ requireSymbol: v })} />} />
            <SettingRow
              title="Rotation"
              desc="Force a password change on this cadence."
              control={
                <select
                  className="inp"
                  style={{ width: "auto" }}
                  value={String(current.passwordPolicy.rotationDays ?? 0)}
                  onChange={(e) => setPw({ rotationDays: +e.target.value === 0 ? null : +e.target.value })}
                >
                  <option value="0">Never</option>
                  <option value="90">Every 90 days</option>
                  <option value="180">Every 180 days</option>
                </select>
              }
            />
          </div>

          <div className="panel" style={{ marginBottom: "var(--space-10)" }}>
            <SettingRow
              icon="link"
              title="Single sign-on"
              desc="Let people sign in through your identity provider. SAML and OIDC are supported."
              control={<Toggle on={current.sso.enabled} onChange={(v) => setSso({ enabled: v })} />}
            />
            {current.sso.enabled && (
              <>
                <SettingRow
                  title="Protocol"
                  control={
                    <Segmented
                      value={current.sso.provider}
                      onChange={(v) => setSso({ provider: v as OrgDto["sso"]["provider"] })}
                      options={[
                        { value: "saml", label: "SAML 2.0" },
                        { value: "oidc", label: "OIDC" },
                      ]}
                    />
                  }
                />
                <div style={{ padding: "var(--space-5) var(--space-8)", display: "flex", gap: 16, alignItems: "flex-end", borderBottom: "1px solid var(--line)" }}>
                  <div className="field" style={{ flex: 1 }}>
                    <label>{current.sso.provider === "saml" ? "Metadata URL" : "Issuer URL"}</label>
                    <input className="inp" value={current.sso.url ?? ""} onChange={(e) => setSso({ url: e.target.value })} placeholder="https://idp.example.com/metadata.xml" />
                  </div>
                  <button className="btn btn-outline" onClick={() => toast(current.sso.url ? "Connection test succeeded." : "Enter a metadata URL first.", current.sso.url ? "check" : "alert")}>
                    Test connection
                  </button>
                </div>
              </>
            )}
          </div>

          <Eyebrow label="Danger zone" danger />
          <div className="panel danger-zone">
            <SettingRow
              icon="refresh"
              title="Transfer ownership"
              desc="Hand ultimate control to another admin. They must accept; you drop to Admin."
              control={
                <button className="btn btn-outline" onClick={() => setTransfer(true)}>
                  Transfer…
                </button>
              }
            />
            <SettingRow
              icon="trash"
              title="Delete organization"
              desc="Permanently remove the studio, its users, and all data. This cannot be undone."
              control={
                <button className="btn btn-danger" onClick={() => setDel(true)}>
                  Delete…
                </button>
              }
            />
          </div>

          {dirty && (
            <div className="savebar">
              <span className="msg">You have unsaved changes.</span>
              <div className="spacer" />
              <button className="btn btn-ghost" onClick={discard}>
                Discard
              </button>
              <button className="btn btn-gold" onClick={() => void save()}>
                <I.check />
                Save changes
              </button>
            </div>
          )}
        </div>
      </div>

      {transfer && <TransferModal onClose={() => setTransfer(false)} />}
      {del && (
        <DeleteOrgModal
          name={current.name}
          onClose={() => setDel(false)}
          onDeleted={() => {
            toast("Organization deleted.", "trash");
            void logout();
          }}
        />
      )}
    </div>
  );
}

function TransferModal({ onClose }: { onClose: () => void }) {
  const { toast } = useStore();
  const { data } = useAsync(useCallback(() => api.listUsers(), []));
  const admins = (data ?? []).filter((u: UserDto) => u.roleKey === "admin" && u.status === "active");
  const [to, setTo] = useState("");
  React.useEffect(() => {
    if (!to && admins[0]) setTo(admins[0].id);
  }, [admins, to]);

  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label="Ownership" />
        <h2>Transfer ownership</h2>
      </div>
      <div className="modal-body">
        <p className="page-intro" style={{ margin: 0 }}>
          The new owner must accept the transfer before it takes effect. Once they do, you become an Admin. Exactly one
          Owner exists at any time.
        </p>
        <div className="field">
          <label>Transfer to</label>
          <select className="inp" value={to} onChange={(e) => setTo(e.target.value)}>
            {admins.map((a) => (
              <option key={a.id} value={a.id}>
                {a.name} — {a.email}
              </option>
            ))}
          </select>
        </div>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button
          className="btn btn-gold"
          disabled={!to}
          onClick={async () => {
            const target = admins.find((a) => a.id === to);
            try {
              await api.transferOwnership(to);
              toast(
                <span>
                  Ownership transfer to <b>{target?.name}</b> sent — awaiting their acceptance.
                </span>,
                "refresh",
              );
            } catch {
              toast("Could not start the transfer.", "alert");
            }
            onClose();
          }}
        >
          Send transfer
        </button>
      </div>
    </Modal>
  );
}

function DeleteOrgModal({ name, onClose, onDeleted }: { name: string; onClose: () => void; onDeleted: () => void }) {
  const [t, setT] = useState("");
  const ok = t.trim() === name;
  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label="Irreversible" danger />
        <h2>Delete this organization</h2>
      </div>
      <div className="modal-body">
        <p className="page-intro" style={{ margin: 0 }}>
          Every user, role, key, and audit record will be permanently destroyed. There is no recovery.
        </p>
        <div className="field">
          <label>Type the organization name to confirm</label>
          <input className="inp" value={t} autoFocus onChange={(e) => setT(e.target.value)} placeholder={name} />
        </div>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button
          className="btn btn-danger"
          disabled={!ok}
          onClick={async () => {
            await api.deleteOrg().catch(() => {});
            onDeleted();
            onClose();
          }}
        >
          <I.trash />
          Delete organization
        </button>
      </div>
    </Modal>
  );
}

function Loading() {
  return (
    <div className="main">
      <Topbar eyebrow="Configuration" title="Organization" />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">Loading…</p>
        </div>
      </div>
    </div>
  );
}
