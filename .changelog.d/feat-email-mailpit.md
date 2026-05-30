---
type: Added
---

- Outbound email: invites and manually-created users now receive a notification
  email. Delivery goes over SMTP (a dockerized Mailpit sink in dev, viewable at
  http://localhost:8026); set `SMTP_HOST` empty to disable. Sending is best-effort
  — a delivery failure never blocks the invite/create.
