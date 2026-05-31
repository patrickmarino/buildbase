---
type: Changed
---

- Bootstrap scopes the Docker network, volume, and containers per project via `COMPOSE_PROJECT_NAME` and now requires a project name, so multiple projects on one host no longer clash on container names or ports.
