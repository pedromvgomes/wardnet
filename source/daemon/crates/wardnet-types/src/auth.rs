use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Access role for API requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Public,
}

/// Identity and authorization context for the current request.
///
/// Set by API middleware and made available to services via
/// `tokio::task_local!`. Admin endpoints populate [`Admin`](Self::Admin),
/// unauthenticated self-service endpoints populate [`Device`](Self::Device)
/// with the caller's MAC address, and requests with no identified caller
/// use [`Anonymous`](Self::Anonymous).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthContext {
    /// Authenticated admin user.
    Admin {
        /// The UUID of the authenticated admin account.
        admin_id: Uuid,
    },
    /// Self-service caller identified by their device MAC address.
    Device {
        /// The MAC address of the caller's device.
        mac: String,
    },
    /// No identity resolved (e.g. unknown IP, public info endpoints).
    Anonymous,
}

impl AuthContext {
    /// Returns `true` if the context represents an admin.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin { .. })
    }

    /// Returns the device MAC if this is a [`Device`](Self::Device) context.
    #[must_use]
    pub fn device_mac(&self) -> Option<&str> {
        match self {
            Self::Device { mac } => Some(mac),
            _ => None,
        }
    }
}

/// An authenticated admin session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub admin_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// A stored API key record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub label: String,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_round_trip() {
        for role in [Role::Admin, Role::Public] {
            let json = serde_json::to_string(&role).unwrap();
            let back: Role = serde_json::from_str(&json).unwrap();
            assert_eq!(role, back);
        }
    }

    #[test]
    fn session_round_trip() {
        let session = Session {
            id: Uuid::nil(),
            admin_id: Uuid::nil(),
            token_hash: "abc123".to_owned(),
            created_at: "2026-03-07T00:00:00Z".parse().unwrap(),
            expires_at: "2026-03-08T00:00:00Z".parse().unwrap(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, back.id);
        assert_eq!(session.token_hash, back.token_hash);
    }

    #[test]
    fn api_key_record_round_trip() {
        let record = ApiKeyRecord {
            id: Uuid::nil(),
            label: "CI key".to_owned(),
            key_hash: "hash123".to_owned(),
            created_at: "2026-03-07T00:00:00Z".parse().unwrap(),
            last_used_at: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        let back: ApiKeyRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record.id, back.id);
        assert_eq!(record.label, back.label);
        assert!(back.last_used_at.is_none());
    }
}
