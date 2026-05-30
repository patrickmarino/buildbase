//! Password-policy validation. Pure: takes a candidate password and the org's
//! [`PasswordPolicy`] and returns every rule it fails (so the UI can show all at
//! once), matching the prototype's Organization → password-policy controls.

use crate::entities::organization::PasswordPolicy;
use crate::error::PolicyViolation;

/// Validate `pw` against `policy`. `Ok(())` when it satisfies every rule;
/// otherwise the full list of violations.
pub fn validate_password(pw: &str, policy: &PasswordPolicy) -> Result<(), Vec<PolicyViolation>> {
    let mut violations = Vec::new();

    let len = pw.chars().count();
    if len < policy.min_length as usize {
        violations.push(PolicyViolation::TooShort {
            min: policy.min_length,
            actual: len,
        });
    }
    if policy.require_number && !pw.chars().any(|c| c.is_ascii_digit()) {
        violations.push(PolicyViolation::MissingNumber);
    }
    if policy.require_symbol && !pw.chars().any(is_symbol) {
        violations.push(PolicyViolation::MissingSymbol);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// A symbol is any non-alphanumeric, non-whitespace character.
fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(min: u8, num: bool, sym: bool) -> PasswordPolicy {
        PasswordPolicy {
            min_length: min,
            require_number: num,
            require_symbol: sym,
            rotation_days: None,
        }
    }

    #[test]
    fn accepts_compliant_password() {
        assert!(validate_password("Sufficient1!", &policy(8, true, true)).is_ok());
    }

    #[test]
    fn flags_too_short() {
        let err = validate_password("aB1!", &policy(8, true, true)).unwrap_err();
        assert!(err.contains(&PolicyViolation::TooShort { min: 8, actual: 4 }));
    }

    #[test]
    fn flags_missing_number_and_symbol() {
        let err = validate_password("abcdefghij", &policy(8, true, true)).unwrap_err();
        assert!(err.contains(&PolicyViolation::MissingNumber));
        assert!(err.contains(&PolicyViolation::MissingSymbol));
        assert_eq!(err.len(), 2);
    }

    #[test]
    fn number_not_required_when_policy_off() {
        assert!(validate_password("abcdefghij", &policy(8, false, false)).is_ok());
    }

    #[test]
    fn counts_unicode_by_char_not_byte() {
        // "néé1!" is 5 chars but more than 5 bytes — length must be by chars.
        assert!(validate_password("néé1!", &policy(5, true, true)).is_ok());
        let err = validate_password("néé1!", &policy(6, false, false)).unwrap_err();
        assert!(err.contains(&PolicyViolation::TooShort { min: 6, actual: 5 }));
    }

    #[test]
    fn reports_all_violations_at_once() {
        let err = validate_password("ab", &policy(8, true, true)).unwrap_err();
        assert_eq!(err.len(), 3);
    }
}
