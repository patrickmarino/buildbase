---
type: Added
---

- Accept-invite / set-password flow: invitation emails now carry a single-use,
  7-day-expiring link (`/accept-invite?token=…`). The invited user opens it, sets a
  password against the org's policy, and is activated and signed in — no admin-set
  password needed. Tokens are stored only as a SHA-256 hash, are single-use, and are
  cleared once consumed.
