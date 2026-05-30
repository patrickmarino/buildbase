---
type: Added
---

- Self-serve password reset: a "Forgot password?" link emails a single-use,
  1-hour-expiring link (`/reset-password?token=…`). The user sets a new password
  against the org's policy and is signed in. Completing a reset revokes every
  existing session. Reset tokens are stored only as a SHA-256 hash and are
  single-use. The request endpoint always responds identically whether or not the
  email matches an account, so it can't be used to enumerate users.
