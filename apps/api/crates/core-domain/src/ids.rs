//! Strongly-typed identifiers. Newtypes over `Uuid` so a `UserId` can never be
//! passed where a `RoleId` is expected, and an opaque random `SessionId`.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! uuid_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            #[must_use]
            pub const fn from_uuid(u: Uuid) -> Self {
                Self(u)
            }
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }
    };
}

uuid_id!(
    /// Identifies a [`crate::entities::user::User`].
    UserId
);
uuid_id!(
    /// Identifies a [`crate::entities::role::Role`].
    RoleId
);
uuid_id!(
    /// Identifies an [`crate::entities::organization::Organization`].
    OrgId
);
uuid_id!(
    /// Identifies an [`crate::entities::api_key::ApiKey`].
    ApiKeyId
);
uuid_id!(
    /// Identifies an [`crate::entities::audit::AuditEvent`].
    AuditEventId
);

/// An opaque, high-entropy session identifier. This is the value stored in the
/// `core_sid` cookie; it is *not* a UUID (it comes from a CSPRNG token), so it
/// is its own newtype over a `String`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub String);

impl SessionId {
    #[must_use]
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
