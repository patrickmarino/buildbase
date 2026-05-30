// Sign-in screen — the one page added on top of the prototype. Editorial card
// on the cream canvas, matching the MadeSpace brand.

import React, { useState } from "react";
import { useStore } from "../store/AppContext";
import { ApiError } from "../lib/api";
import { I } from "../components/icons";

export function LoginPage() {
  const { login } = useStore();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr("");
    setBusy(true);
    try {
      await login(email.trim(), password);
    } catch (ex) {
      setErr(ex instanceof ApiError && ex.status === 401 ? "Those credentials don’t match." : "Sign-in failed. Try again.");
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
          <span>Sign in</span>
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
          Welcome back.
        </h1>

        <form onSubmit={submit} style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
          <div className="field">
            <label htmlFor="email">Email address</label>
            <input
              id="email"
              className={"inp" + (err ? " bad" : "")}
              type="email"
              autoFocus
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="you@madespace.co"
            />
          </div>
          <div className="field">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              className={"inp" + (err ? " bad" : "")}
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
            />
            {err ? (
              <div className="hint err">{err}</div>
            ) : (
              <div className="hint">Use the credentials seeded by the API on first boot.</div>
            )}
          </div>
          <button className="btn btn-gold" type="submit" disabled={busy} style={{ marginTop: "var(--space-2)" }}>
            <I.shield />
            {busy ? "Signing in…" : "Sign in"}
          </button>
        </form>
      </div>
    </div>
  );
}
