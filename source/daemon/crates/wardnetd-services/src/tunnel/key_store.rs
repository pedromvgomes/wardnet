//! Tunnel-module private key-store facade.
//!
//! The daemon stores `WireGuard` private keys in the general-purpose
//! [`SecretStore`] alongside backup passphrases and destination
//! credentials. `TunnelService` never talks to the secret store directly —
//! it goes through the narrower [`KeyStore`] trait defined here, which
//! only exposes the operations tunnels care about (save, load, delete,
//! keyed by tunnel UUID).
//!
//! Keeping both the trait and its adapter local to the tunnel module
//! means the rest of the codebase only ever sees `SecretStore` — no other
//! component can accidentally couple to the narrower interface.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;
use wardnetd_data::secret_store::SecretStore;

/// Namespace used for `WireGuard` private keys inside the secret store.
const WIREGUARD_NAMESPACE: &str = "wireguard";

/// Narrow interface for reading and writing `WireGuard` private keys.
///
/// Private keys never appear in API responses, logs, or the database.
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Save a private key for the given tunnel.
    async fn save_key(&self, tunnel_id: &Uuid, private_key: &str) -> anyhow::Result<()>;

    /// Load the private key for the given tunnel.
    async fn load_key(&self, tunnel_id: &Uuid) -> anyhow::Result<String>;

    /// Delete the private key for the given tunnel.
    async fn delete_key(&self, tunnel_id: &Uuid) -> anyhow::Result<()>;
}

/// Adapts a [`SecretStore`] to the narrower [`KeyStore`] interface.
///
/// Keys are stored at `wireguard/<tunnel-uuid>.key` under the store root.
pub struct KeyStoreAdapter {
    store: Arc<dyn SecretStore>,
}

impl KeyStoreAdapter {
    /// Wrap a shared [`SecretStore`] as a [`KeyStore`].
    #[must_use]
    pub fn new(store: Arc<dyn SecretStore>) -> Self {
        Self { store }
    }

    fn path(tunnel_id: &Uuid) -> String {
        format!("{WIREGUARD_NAMESPACE}/{tunnel_id}.key")
    }
}

#[async_trait]
impl KeyStore for KeyStoreAdapter {
    async fn save_key(&self, tunnel_id: &Uuid, private_key: &str) -> anyhow::Result<()> {
        self.store
            .put(&Self::path(tunnel_id), private_key.as_bytes())
            .await?;
        tracing::debug!(tunnel_id = %tunnel_id, "saved private key: tunnel_id={tunnel_id}");
        Ok(())
    }

    async fn load_key(&self, tunnel_id: &Uuid) -> anyhow::Result<String> {
        let bytes = self
            .store
            .get(&Self::path(tunnel_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("private key not found for tunnel_id={tunnel_id}"))?;
        String::from_utf8(bytes).map_err(|e| {
            anyhow::anyhow!("private key is not valid utf-8: tunnel_id={tunnel_id}: {e}")
        })
    }

    async fn delete_key(&self, tunnel_id: &Uuid) -> anyhow::Result<()> {
        self.store.delete(&Self::path(tunnel_id)).await?;
        tracing::debug!(tunnel_id = %tunnel_id, "deleted private key: tunnel_id={tunnel_id}");
        Ok(())
    }
}
