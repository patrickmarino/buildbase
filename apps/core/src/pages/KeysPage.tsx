// API Keys / service accounts — list, create (token shown once), revoke.

import { useCallback, useState } from "react";
import { Topbar } from "../components/Shell";
import { Modal, Confirm } from "../components/Modal";
import { StatusPill, Eyebrow } from "../components/ui";
import { I } from "../components/icons";
import { api, ApiError } from "../lib/api";
import { useAsync } from "../lib/useAsync";
import type { ApiKeyDto } from "../lib/types";
import { useStore } from "../store/AppContext";

const SCOPE_OPTIONS = [
  { id: "users.view", label: "Read users" },
  { id: "users.edit", label: "Edit users" },
  { id: "audit.view", label: "Read audit log" },
  { id: "audit.export", label: "Export audit log" },
  { id: "roles.matrix", label: "Read permission matrix" },
  { id: "org.branding", label: "Read org profile" },
];

export function KeysPage() {
  const { toast } = useStore();
  const load = useCallback(() => api.listKeys(), []);
  const { data, loading, reload } = useAsync(load);
  const [create, setCreate] = useState(false);
  const [revoke, setRevoke] = useState<ApiKeyDto | null>(null);

  if (loading || !data) return <Loading />;

  const doRevoke = async (k: ApiKeyDto) => {
    try {
      await api.revokeKey(k.id);
      await reload();
      toast(
        <span>
          Key <b>{k.name}</b> revoked — it can no longer authenticate.
        </span>,
        "shieldOff",
      );
    } catch (e) {
      toast(e instanceof ApiError ? e.message : "Could not revoke key.", "alert");
    }
  };

  return (
    <div className="main">
      <Topbar
        eyebrow="Service accounts"
        title="API Keys"
        actions={
          <button className="btn btn-gold" onClick={() => setCreate(true)}>
            <I.plus />
            New key
          </button>
        }
      />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">
            Non-human actors — integrations, jobs, and scripts. Each key is a scoped token with no interactive login.
            Grant the narrowest set of permissions it needs, and revoke the moment it’s unused.
          </p>

          <div className="panel">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Service account</th>
                  <th>Token</th>
                  <th>Scopes</th>
                  <th>Created</th>
                  <th>Status</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {data.map((k) => (
                  <tr key={k.id} style={k.status === "revoked" ? { opacity: 0.55 } : undefined}>
                    <td>
                      <div className="uname">
                        <div style={{ width: 34, height: 34, borderRadius: "var(--r)", background: "var(--surface-mute)", display: "grid", placeItems: "center", color: "var(--accent-strong)", flex: "none" }}>
                          <I.key style={{ width: 17, height: 17, strokeWidth: 1.5 }} />
                        </div>
                        <span className="nm">{k.name}</span>
                      </div>
                    </td>
                    <td>
                      <span className="mono">{k.prefix}••••••</span>
                    </td>
                    <td>
                      <span style={{ display: "inline-flex", gap: 6, flexWrap: "wrap" }}>
                        {k.scopes.map((sc) => (
                          <span key={sc} className="rchip" style={{ padding: "3px 9px", fontSize: "9.5px" }}>
                            {sc}
                          </span>
                        ))}
                      </span>
                    </td>
                    <td>
                      <span className="mono">{k.created.slice(0, 10)}</span>
                    </td>
                    <td>
                      <StatusPill status={k.status} />
                    </td>
                    <td style={{ textAlign: "right" }}>
                      {k.status === "active" ? (
                        <button className="btn btn-ghost btn-sm row-act" onClick={() => setRevoke(k)}>
                          Revoke
                        </button>
                      ) : (
                        <span className="mono" style={{ fontSize: 11 }}>
                          —
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {create && <NewKeyModal onClose={() => setCreate(false)} onDone={() => reload()} />}
      {revoke && (
        <Confirm
          title={`Revoke ${revoke.name}?`}
          danger
          confirmLabel="Revoke key"
          body="Any integration using this token will immediately fail to authenticate. This cannot be undone — you would need to issue a new key."
          onConfirm={() => void doRevoke(revoke)}
          onClose={() => setRevoke(null)}
        />
      )}
    </div>
  );
}

function NewKeyModal({ onClose, onDone }: { onClose: () => void; onDone: () => void }) {
  const { toast } = useStore();
  const [step, setStep] = useState<1 | 2>(1);
  const [name, setName] = useState("");
  const [scopes, setScopes] = useState<string[]>([]);
  const [err, setErr] = useState("");
  const [token, setToken] = useState("");
  const [copied, setCopied] = useState(false);

  const toggle = (id: string) => setScopes((s) => (s.includes(id) ? s.filter((x) => x !== id) : [...s, id]));

  const generate = async () => {
    if (!name.trim()) {
      setErr("Name the service account.");
      return;
    }
    if (scopes.length === 0) {
      setErr("Select at least one scope.");
      return;
    }
    try {
      const created = await api.createKey(name.trim(), scopes);
      setToken(created.token);
      setStep(2);
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : "Could not create key.");
    }
  };

  const copy = () => {
    void navigator.clipboard?.writeText(token).catch(() => {});
    setCopied(true);
    toast("Token copied to clipboard.", "copy");
  };

  if (step === 2) {
    return (
      <Modal onClose={onClose} wide>
        <div className="modal-head">
          <Eyebrow label="Save your token" />
          <h2>{name} is ready</h2>
        </div>
        <div className="modal-body">
          <p className="page-intro" style={{ margin: 0 }}>
            Copy this token now — for security it is shown once and never again. Store it in your secrets manager.
          </p>
          <div style={{ display: "flex", alignItems: "center", gap: 12, background: "var(--ms-charcoal-900)", borderRadius: "var(--r)", padding: "14px 16px" }}>
            <span className="mono" style={{ color: "#F2EFE9", fontSize: 13, flex: 1, wordBreak: "break-all", letterSpacing: 0 }}>
              {token}
            </span>
            <button className={"btn " + (copied ? "btn-outline" : "btn-gold") + " btn-sm"} style={copied ? { borderColor: "var(--accent)", color: "var(--accent)" } : undefined} onClick={copy}>
              {copied ? <I.check /> : <I.copy />}
              {copied ? "Copied" : "Copy"}
            </button>
          </div>
          <div style={{ display: "flex", alignItems: "flex-start", gap: 10, color: "var(--ink-muted)" }}>
            <I.alert style={{ width: 16, height: 16, strokeWidth: 1.5, color: "var(--accent-strong)", flex: "none", marginTop: 2 }} />
            <span style={{ fontFamily: "var(--font-body)", fontSize: 12.5, lineHeight: 1.5 }}>
              Treat it like a password. Never commit it to source control or log it. Revoke immediately if exposed.
            </span>
          </div>
        </div>
        <div className="modal-foot">
          <button
            className="btn btn-gold"
            onClick={() => {
              toast(
                <span>
                  Service account <b>{name}</b> created.
                </span>,
                "key",
              );
              onDone();
              onClose();
            }}
          >
            Done
          </button>
        </div>
      </Modal>
    );
  }

  return (
    <Modal onClose={onClose} wide>
      <div className="modal-head">
        <Eyebrow label="New service account" />
        <h2>Create an API key</h2>
      </div>
      <div className="modal-body">
        <div className="field">
          <label>Name</label>
          <input
            className={"inp" + (err && !name.trim() ? " bad" : "")}
            value={name}
            autoFocus
            onChange={(e) => {
              setName(e.target.value);
              setErr("");
            }}
            placeholder="e.g. Booking sync"
          />
          <div className="hint">What this integration is for — visible in the audit log.</div>
        </div>
        <div className="field">
          <label>Scopes — least privilege</label>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
            {SCOPE_OPTIONS.map((o) => (
              <div key={o.id} className={"checkrow" + (scopes.includes(o.id) ? " on" : "")} onClick={() => toggle(o.id)}>
                <span className="cbox">{scopes.includes(o.id) && <I.check />}</span>
                <span className="crl">{o.label}</span>
                <span className="crs">{o.id}</span>
              </div>
            ))}
          </div>
          {err && scopes.length === 0 ? (
            <div className="hint err">{err}</div>
          ) : (
            <div className="hint">A key can never exceed the permissions of an Admin. Deny-by-default applies to everything unticked.</div>
          )}
        </div>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button className="btn btn-gold" onClick={generate}>
          <I.key />
          Generate key
        </button>
      </div>
    </Modal>
  );
}

function Loading() {
  return (
    <div className="main">
      <Topbar eyebrow="Service accounts" title="API Keys" />
      <div className="scroll">
        <div className="page">
          <p className="page-intro">Loading…</p>
        </div>
      </div>
    </div>
  );
}
