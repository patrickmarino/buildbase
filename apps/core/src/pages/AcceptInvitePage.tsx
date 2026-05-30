// Accept-invite / set-password screen. Reached via the link in the invitation
// email (`/accept-invite?token=…`). The new hire chooses a password; on success
// the account is activated and signed in. Mirrors the LoginPage editorial card.

import React, { useState } from "react";
import { useStore } from "../store/AppContext";
import { ApiError } from "../lib/api";
import { I } from "../components/icons";

export function AcceptInvitePage({ token }: { token: string }) {
  const { acceptInvite } = useStore();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr("");
    if (password !== confirm) {
      setErr("Those passwords don’t match.");
      return;
    }
    setBusy(true);
    try {
      await acceptInvite(token, password);
      // On success the store sets `me` and the app shell takes over; tidy the URL.
      window.history.replaceState(null, "", "/");
    } catch (ex) {
      if (ex instanceof ApiError && ex.status === 401) {
        setErr("This invitation is invalid or has expired. Ask an admin to re-send it.");
      } else if (ex instanceof ApiError && ex.status === 422) {
        setErr(ex.message || "That password doesn’t meet the organization’s policy.");
      } else {
        setErr("Couldn’t set your password. Try again.");
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        display: "grid",
        placeItems: "center",
        background: "var(--ms-gold-50)",
        padding: 24,
      }}
    >
      <div
        style={{
          width: "100%",
          maxWidth: 420,
          background: "#fff",
          border: "1px solid var(--ms-divider)",
          borderRadius: "var(--radius-lg)",
          boxShadow: "var(--shadow-3)",
          padding: "var(--space-10)",
        }}
      >
        <div className="brand" style={{ padding: 0, marginBottom: "var(--space-8)" }}>
          <img src="/assets/madespace-icon.png" alt="" style={{ width: 40, height: 40 }} />
          <div
            className="wm"
            style={{ color: "var(--ms-charcoal)", fontFamily: "var(--font-display)", fontWeight: 700, fontSize: 22 }}
          >
            made space
            <small
              style={{
                display: "block",
                fontFamily: "var(--font-heading)",
                fontSize: 9.5,
                letterSpacing: "0.34em",
                color: "var(--ms-gold-600)",
                marginTop: 5,
                textTransform: "uppercase",
              }}
            >
              Core · Admin
            </small>
          </div>
        </div>

        <div className="sec-eyebrow">
          <span className="rule" />
          <span>Accept invitation</span>
        </div>
        <h1
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 700,
            fontSize: 28,
            margin: "0 0 var(--space-8)",
            color: "var(--ink)",
            letterSpacing: "-0.01em",
          }}
        >
          Set your password.
        </h1>

        <form onSubmit={submit} style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
          <div className="field">
            <label htmlFor="ai-password">New password</label>
            <input
              id="ai-password"
              className={"inp" + (err ? " bad" : "")}
              type="password"
              autoFocus
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
            />
          </div>
          <div className="field">
            <label htmlFor="ai-confirm">Confirm password</label>
            <input
              id="ai-confirm"
              className={"inp" + (err ? " bad" : "")}
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              placeholder="••••••••"
            />
            {err ? (
              <div className="hint err">{err}</div>
            ) : (
              <div className="hint">Choose a strong password to finish setting up your account.</div>
            )}
          </div>
          <button className="btn btn-gold" type="submit" disabled={busy} style={{ marginTop: "var(--space-2)" }}>
            <I.shield />
            {busy ? "Setting password…" : "Set password & sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
