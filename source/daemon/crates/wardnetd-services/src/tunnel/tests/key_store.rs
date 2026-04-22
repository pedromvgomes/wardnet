//! Tests for the [`KeyStoreAdapter`] bridge between [`SecretStore`]
//! and the narrower [`KeyStore`] trait `TunnelService` depends on.
//!
//! A real [`FileSecretStore`] is used (over a tempdir) so the
//! `wireguard/<tunnel-uuid>.key` layout is exercised end-to-end.

use std::sync::Arc;

use uuid::Uuid;
use wardnetd_data::secret_store::{FileSecretStore, SecretStore};

use crate::tunnel::key_store::{KeyStore, KeyStoreAdapter};

fn fresh_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("wardnet-key-adapter-test-{}", Uuid::new_v4()))
}

#[tokio::test]
async fn save_then_load_returns_same_key() {
    let root = fresh_root();
    let store: Arc<dyn SecretStore> = Arc::new(FileSecretStore::new(root.clone()));
    let adapter = KeyStoreAdapter::new(store);

    let tunnel_id = Uuid::new_v4();
    adapter
        .save_key(&tunnel_id, "abc123-private-key")
        .await
        .unwrap();

    let loaded = adapter.load_key(&tunnel_id).await.unwrap();
    assert_eq!(loaded, "abc123-private-key");

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn save_writes_to_wireguard_namespace() {
    let root = fresh_root();
    let store_arc: Arc<dyn SecretStore> = Arc::new(FileSecretStore::new(root.clone()));
    let adapter = KeyStoreAdapter::new(store_arc.clone());

    let tunnel_id = Uuid::new_v4();
    adapter.save_key(&tunnel_id, "k").await.unwrap();

    // Paths are `wireguard/<uuid>.key` under the store root.
    let listed = store_arc.list("wireguard").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0], format!("wireguard/{tunnel_id}.key"));

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn load_missing_key_errors() {
    let root = fresh_root();
    let store: Arc<dyn SecretStore> = Arc::new(FileSecretStore::new(root.clone()));
    let adapter = KeyStoreAdapter::new(store);

    let err = adapter.load_key(&Uuid::new_v4()).await.unwrap_err();
    assert!(
        format!("{err:#}").contains("private key not found"),
        "unexpected error: {err}"
    );

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn load_non_utf8_key_errors() {
    let root = fresh_root();
    let store_arc: Arc<dyn SecretStore> = Arc::new(FileSecretStore::new(root.clone()));
    let adapter = KeyStoreAdapter::new(store_arc.clone());

    // Drop invalid UTF-8 bytes into the adapter's expected path and
    // confirm `load_key` surfaces a readable error instead of panicking.
    let tunnel_id = Uuid::new_v4();
    store_arc
        .put(
            &format!("wireguard/{tunnel_id}.key"),
            &[0xFF, 0xFE, 0xFD, 0xFC],
        )
        .await
        .unwrap();

    let err = adapter.load_key(&tunnel_id).await.unwrap_err();
    assert!(
        format!("{err:#}").contains("not valid utf-8"),
        "unexpected error: {err}"
    );

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn delete_removes_key_from_store() {
    let root = fresh_root();
    let store: Arc<dyn SecretStore> = Arc::new(FileSecretStore::new(root.clone()));
    let adapter = KeyStoreAdapter::new(store);

    let tunnel_id = Uuid::new_v4();
    adapter.save_key(&tunnel_id, "k").await.unwrap();
    adapter.delete_key(&tunnel_id).await.unwrap();

    let err = adapter.load_key(&tunnel_id).await.unwrap_err();
    assert!(format!("{err:#}").contains("private key not found"));

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn delete_is_idempotent_for_missing_keys() {
    let root = fresh_root();
    let store: Arc<dyn SecretStore> = Arc::new(FileSecretStore::new(root.clone()));
    let adapter = KeyStoreAdapter::new(store);

    // Delete without prior save should succeed — matches FileSecretStore's
    // delete semantics.
    adapter.delete_key(&Uuid::new_v4()).await.unwrap();

    let _ = tokio::fs::remove_dir_all(&root).await;
}
