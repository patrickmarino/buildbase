//! # core-domain
//!
//! The pure heart of Core. Entities, value objects, repository **ports** (traits),
//! and the business rules that everything else trusts. This crate has **no**
//! dependency on axum, sqlx, or tokio — clean architecture is enforced by the
//! dependency list in `Cargo.toml`, not merely by convention.
//!
//! The high-value, heavily-tested logic lives in [`services`]:
//! authorization ([`services::authz::can`], deny-by-default), the permission
//! matrix rules ([`services::matrix_rules`]), the role guards
//! ([`services::role_guards`]), and password-policy validation
//! ([`services::password_policy`]). These are plain synchronous functions over
//! already-loaded data, so they are trivial to unit-test in isolation.

// The domain enums expose ergonomic inherent `from_str` (and a matching
// `as_str`) helpers; these intentionally don't implement `std::str::FromStr`.
#![allow(clippy::should_implement_trait)]

pub mod entities;
pub mod error;
pub mod ids;
pub mod ports;
pub mod seed;
pub mod services;

pub use error::DomainError;
