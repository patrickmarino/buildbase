-- Core schema. The role catalog, permission matrix, default org, and Owner are
-- seeded programmatically at startup (see core-infra::seed) from the canonical
-- core_domain::seed data, so this migration only defines structure.

-- ── organizations ───────────────────────────────────────────
-- owner_id / pending_owner_id reference users but carry no DB FK (the org and
-- its first user are created together; the app maintains the link).
create table organizations (
    id                uuid primary key,
    name              text not null,
    domain            text,
    owner_id          uuid,
    pending_owner_id  uuid,
    branding_accent   text not null,
    mfa_enabled       boolean not null,
    mfa_method        text not null,
    mfa_enforce       text not null,
    pw_min_length     smallint not null,
    pw_require_number boolean not null,
    pw_require_symbol boolean not null,
    pw_rotation_days  smallint,
    sso_enabled       boolean not null,
    sso_provider      text not null,
    sso_url           text,
    created_at        timestamptz not null default now()
);

-- ── roles ───────────────────────────────────────────────────
create table roles (
    id            uuid primary key,
    org_id        uuid not null references organizations(id) on delete cascade,
    key           text not null,
    name          text not null,
    builtin       text,
    is_column     boolean not null,
    base_role_id  uuid references roles(id) on delete set null,
    rank          smallint not null,
    unique (org_id, key)
);

-- ── users ───────────────────────────────────────────────────
-- Emails are normalized to lowercase by the domain before they arrive here, so
-- a plain unique(org_id, email) is sufficient for case-insensitive uniqueness.
create table users (
    id             uuid primary key,
    org_id         uuid not null references organizations(id) on delete cascade,
    email          text not null,
    name           text not null,
    role_id        uuid not null references roles(id),
    status         text not null,
    scope          text,
    password_hash  text,
    created_at     timestamptz not null default now(),
    last_active_at timestamptz,
    unique (org_id, email)
);
create index users_org_status_idx on users (org_id, status);

-- ── permission matrix cells ─────────────────────────────────
create table permission_cells (
    org_id      uuid not null references organizations(id) on delete cascade,
    role_id     uuid not null references roles(id) on delete cascade,
    action_key  text not null,
    state       text not null,
    primary key (role_id, action_key)
);

-- ── sessions ────────────────────────────────────────────────
create table sessions (
    id         text primary key,
    user_id    uuid not null references users(id) on delete cascade,
    created_at timestamptz not null,
    expires_at timestamptz not null
);
create index sessions_user_idx on sessions (user_id);

-- ── audit events (append-only) ──────────────────────────────
create table audit_events (
    id             uuid primary key,
    org_id         uuid not null references organizations(id) on delete cascade,
    ts             timestamptz not null,
    actor_id       uuid not null,
    actor_name     text not null,
    actor_role_key text not null,
    action         text not null,
    category       text not null,
    target         text,
    before_val     text,
    after_val      text,
    ip             text
);
create index audit_org_ts_idx on audit_events (org_id, ts desc);
create index audit_org_cat_idx on audit_events (org_id, category);

-- ── service-account API keys ────────────────────────────────
create table api_keys (
    id           uuid primary key,
    org_id       uuid not null references organizations(id) on delete cascade,
    name         text not null,
    prefix       text not null,
    token_hash   text not null,
    scopes       jsonb not null,
    status       text not null,
    created_at   timestamptz not null,
    last_used_at timestamptz
);
create index api_keys_org_idx on api_keys (org_id);
