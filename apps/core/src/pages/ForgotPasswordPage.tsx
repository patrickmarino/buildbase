// Forgot-password screen. Collects an email and asks the API to send a reset
// link. The response is intentionally identical whether or not the email matches
// an account (no enumeration), so we always show the same confirmation.

import React, { useState } from "react";
import { api } from "../lib/api";
import { I } from "../components/icons";

export function ForgotPasswordPage() {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    try {
      await api.requestPasswordReset(email.trim());
    } catch {
      // Swallow errors: we never reveal whether the email exists or the send failed.
    } finally {
      setBusy(false);
      setSent(true);
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
          <span>Forgot password</span>
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
          {sent ? "Check your inbox." : "Reset your password."}
        </h1>

        {sent ? (
          <>
            <p style={{ color: "var(--ink-muted)", margin: "0 0 var(--space-8)", lineHeight: 1.6 }}>
              If an account exists for that address, we’ve sent a link to reset your password. The
              link expires in 1 hour.
            </p>
            <a className="btn btn-ghost" href="/" style={{ textDecoration: "none" }}>
              Back to sign in
            </a>
          </>
        ) : (
          <form onSubmit={submit} style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
            <div className="field">
              <label htmlFor="fp-email">Email address</label>
              <input
                id="fp-email"
                className="inp"
                type="email"
                autoFocus
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="you@madespace.co"
              />
              <div className="hint">We’ll email you a link to choose a new password.</div>
            </div>
            <button className="btn btn-gold" type="submit" disabled={busy} style={{ marginTop: "var(--space-2)" }}>
              <I.shield />
              {busy ? "Sending…" : "Send reset link"}
            </button>
            <a
              href="/"
              style={{ textAlign: "center", color: "var(--ms-gold-600)", fontSize: 13, textDecoration: "none" }}
            >
              Back to sign in
            </a>
          </form>
        )}
      </div>
    </div>
  );
}
