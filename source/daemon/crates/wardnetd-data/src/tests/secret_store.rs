//! Tests for [`FileSecretStore`] and [`NullSecretStore`].
//!
//! The `file_system` provider is the only concrete store today; these
//! tests exercise the full happy-path round-trip plus the error paths
//! that harder-to-reach variants can piggy-back on (rejected paths,
//! missing entries, permission bits).

use uuid::Uuid;
use wardnet_common::config::SecretStoreConfig;

use crate::secret_store::{FileSecretStore, NullSecretStore, SecretStore, build_secret_store};

fn fresh_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("wardnet-test-secrets-{}", Uuid::new_v4()))
}

#[tokio::test]
async fn file_secret_store_put_get_round_trip() {
    let root = fresh_root();
    let store = FileSecretStore::new(root.clone());

    store
        .put("wireguard/example.key", b"super-secret-bytes")
        .await
        .expect("put should succeed");

    let loaded = store
        .get("wireguard/example.key")
        .await
        .expect("get should succeed")
        .expect("value should be present");
    assert_eq!(loaded, b"super-secret-bytes");

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn file_secret_store_missing_path_returns_none() {
    let root = fresh_root();
    let store = FileSecretStore::new(root.clone());

    let result = store.get("wireguard/missing.key").await.unwrap();
    assert!(result.is_none());

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn file_secret_store_delete_removes_entry() {
    let root = fresh_root();
    let store = FileSecretStore::new(root.clone());

    store.put("wireguard/delete-me", b"bytes").await.unwrap();
    store.delete("wireguard/delete-me").await.unwrap();
    assert!(store.get("wireguard/delete-me").await.unwrap().is_none());

    // Deleting a missing path is a no-op.
    store.delete("wireguard/already-gone").await.unwrap();

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn file_secret_store_list_returns_all_under_prefix() {
    let root = fresh_root();
    let store = FileSecretStore::new(root.clone());

    store.put("wireguard/a.key", b"a").await.unwrap();
    store.put("wireguard/b.key", b"b").await.unwrap();
    store.put("backup/c", b"c").await.unwrap();

    let mut wg = store.list("wireguard").await.unwrap();
    wg.sort();
    assert_eq!(wg, vec!["wireguard/a.key", "wireguard/b.key"]);

    let mut all = store.list("").await.unwrap();
    all.sort();
    assert_eq!(all, vec!["backup/c", "wireguard/a.key", "wireguard/b.key"]);

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn file_secret_store_rejects_traversal_paths() {
    let root = fresh_root();
    let store = FileSecretStore::new(root.clone());

    // Absolute paths, `..` segments, and empty segments all hard-fail.
    assert!(store.put("/etc/shadow", b"x").await.is_err());
    assert!(store.put("wireguard/../oops", b"x").await.is_err());
    assert!(store.put("wireguard//a", b"x").await.is_err());
    assert!(store.put("", b"x").await.is_err());

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[cfg(unix)]
#[tokio::test]
async fn file_secret_store_writes_owner_only_perms() {
    use std::os::unix::fs::PermissionsExt;

    let root = fresh_root();
    let store = FileSecretStore::new(root.clone());

    store.put("wireguard/perm.key", b"bytes").await.unwrap();
    let meta = tokio::fs::metadata(root.join("wireguard").join("perm.key"))
        .await
        .unwrap();
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn null_secret_store_rejects_all_operations() {
    let store = NullSecretStore;
    assert!(store.put("any/path", b"x").await.is_err());
    assert!(store.get("any/path").await.is_err());
    assert!(store.delete("any/path").await.is_err());
    assert!(store.list("any").await.is_err());
}

#[tokio::test]
async fn build_secret_store_wires_file_system_provider() {
    let root = fresh_root();
    let config = SecretStoreConfig::FileSystem { path: root.clone() };
    let store = build_secret_store(Some(&config));

    store.put("wireguard/wired.key", b"ok").await.unwrap();
    let loaded = store.get("wireguard/wired.key").await.unwrap().unwrap();
    assert_eq!(loaded, b"ok");

    let _ = tokio::fs::remove_dir_all(&root).await;
}

#[tokio::test]
async fn build_secret_store_wires_null_when_config_absent() {
    let store = build_secret_store(None);
    assert!(store.put("wireguard/x", b"y").await.is_err());
}
