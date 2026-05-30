//! Pure domain services — the business rules, as synchronous functions over
//! already-loaded data. These are the primary unit-test targets.

pub mod authz;
pub mod matrix_rules;
pub mod password_policy;
pub mod role_guards;
