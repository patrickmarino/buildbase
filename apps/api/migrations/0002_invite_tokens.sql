-- Single-use, expiring invitation tokens. Only the hash is stored; the plaintext
-- is emailed to the invitee and exchanged at /accept-invite for a password.
create table invite_tokens (
    id          uuid primary key,
    user_id     uuid not null references users(id) on delete cascade,
    token_hash  text not null unique,
    created_at  timestamptz not null,
    expires_at  timestamptz not null
);
create index invite_tokens_user_idx on invite_tokens (user_id);
