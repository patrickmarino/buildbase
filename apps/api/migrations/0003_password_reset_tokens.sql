-- Single-use, expiring password-reset tokens. Only the hash is stored; the
-- plaintext is emailed to the user and exchanged at /reset-password for a new
-- password.
create table password_reset_tokens (
    id          uuid primary key,
    user_id     uuid not null references users(id) on delete cascade,
    token_hash  text not null unique,
    created_at  timestamptz not null,
    expires_at  timestamptz not null
);
create index password_reset_tokens_user_idx on password_reset_tokens (user_id);
