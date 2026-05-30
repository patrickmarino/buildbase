//! # core-infra
//!
//! Adapters that implement the domain ports with real technology: sqlx/Postgres
//! repositories, Argon2 password hashing, a CSPRNG token generator, and the
//! system clock. Plus [`seed::ensure_seeded`], the idempotent first-boot seed.
//!
//! This crate depends on `core-domain` (to implement its traits) and `core-app`
//! (only for its `test-support` fakes in tests). It knows nothing of HTTP.

pub mod argon2_hasher;
pub mod db;
pub mod error;
pub mod rand_token;
pub mod repos;
pub mod seed;
pub mod smtp_mailer;
pub mod sys_clock;

pub use argon2_hasher::Argon2Hasher;
pub use db::{connect, migrate};
pub use rand_token::RandTokenGenerator;
pub use repos::*;
pub use smtp_mailer::{NoopMailer, SmtpMailer};
pub use sys_clock::SystemClock;
