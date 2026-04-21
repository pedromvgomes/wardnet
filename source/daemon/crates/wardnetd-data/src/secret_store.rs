//! Secret store — the daemon's secure storage for credential material.
//!
//! Holds anything that must never cross the wire, enter logs, or land in
//! the `SQLite` database. Today that's `WireGuard` private keys (via the
//! tunnel-module-local `KeyStoreAdapter`); subsequent PRs add backup
//! passphrases and destination credentials (`OneDrive` refresh tokens,
//! `SFTP` keys) alongside them.
//!
//! ### Layout
//!
//! ```text
//! <root>/
//! ├── wireguard/<tunnel-uuid>.key        # WireGuard private keys
//! ├── backup/passphrases/<job-uuid>      # scheduled-backup passphrases (PR 2)
//! └── destinations/<dest-uuid>           # destination creds (PR 3)
//! ```
//!
//! ### Types
//!
//! * [`SecretStore`] is the general-purpose interface: opaque path strings,
//!   byte-array values. Callers use it directly; narrower type-safe facades
//!   (`KeyStoreAdapter` in the tunnel module, similar wrappers in PRs 2–3)
//!   live alongside the consumers that need them.
//! * [`FileSecretStore`] implements `SecretStore` against the local
//!   filesystem with mode-0600 files — the `file_system` provider.
//! * [`NullSecretStore`] is wired when `[secret_store]` is absent from
//!   the config. Every operation fails with a clear error so the daemon
//!   still starts (DHCP/DNS/device-detection keep working) while tunnel
//!   creation and backup features refuse cleanly instead of panicking on
//!   a missing path.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use wardnet_common::config::SecretStoreConfig;

/// A generic secret store keyed by forward-slash-separated paths.
///
/// The path is opaque to the backend — `FileSecretStore` maps each segment
/// to a directory under the root; a future `HashicorpVaultStore` might
/// map it to a KV mount.
///
/// Paths must be ASCII, use `/` as the separator, and contain no `..` or
/// empty segments. Implementations treat a violation as a hard error
/// rather than silently sanitising, because mis-pathed secrets are a sign
/// of a bug, not input to recover from.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Write `value` at `path`, replacing any existing secret.
    async fn put(&self, path: &str, value: &[u8]) -> anyhow::Result<()>;

    /// Read the secret at `path`, or `None` if absent.
    async fn get(&self, path: &str) -> anyhow::Result<Option<Vec<u8>>>;

    /// Remove the secret at `path`. No-op when already absent.
    async fn delete(&self, path: &str) -> anyhow::Result<()>;

    /// List secret paths whose leading segments match `prefix`. Returns
    /// full paths so results can be round-tripped back through [`Self::get`].
    async fn list(&self, prefix: &str) -> anyhow::Result<Vec<String>>;
}

/// Reject paths that would escape the store root or otherwise surprise a
/// filesystem-backed implementation.
fn validate_path(path: &str) -> anyhow::Result<()> {
    if path.is_empty() {
        anyhow::bail!("secret path must not be empty");
    }
    if path.starts_with('/') || path.starts_with('\\') {
        anyhow::bail!("secret path must be relative, got: {path}");
    }
    for segment in path.split('/') {
        if segment.is_empty() {
            anyhow::bail!("secret path must not contain empty segments: {path}");
        }
        if segment == ".." || segment == "." {
            anyhow::bail!("secret path must not contain .. or . segments: {path}");
        }
    }
    Ok(())
}

/// Filesystem-backed [`SecretStore`].
///
/// Secrets are written as mode-0600 files rooted at `root`. Each path
/// segment becomes a directory; the final segment is the file name. The
/// `wardnet` user must own `root` and have write access to it.
#[derive(Debug)]
pub struct FileSecretStore {
    root: PathBuf,
}

impl FileSecretStore {
    /// Create a new file-backed secret store rooted at `root`.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn full_path(&self, rel: &str) -> PathBuf {
        let mut p = self.root.clone();
        for segment in rel.split('/') {
            p.push(segment);
        }
        p
    }

    async fn walk_prefix(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        let root = if prefix.is_empty() {
            self.root.clone()
        } else {
            self.full_path(prefix)
        };
        if !root.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let file_type = entry.file_type().await?;
                if file_type.is_dir() {
                    stack.push(path);
                } else if file_type.is_file() {
                    let rel = path
                        .strip_prefix(&self.root)
                        .map_err(|e| anyhow::anyhow!("path outside secret store root: {e}"))?;
                    out.push(
                        rel.to_string_lossy()
                            .replace(std::path::MAIN_SEPARATOR, "/"),
                    );
                }
            }
        }
        Ok(out)
    }
}

#[async_trait]
impl SecretStore for FileSecretStore {
    async fn put(&self, path: &str, value: &[u8]) -> anyhow::Result<()> {
        validate_path(path)?;
        let full = self.full_path(path);
        if let Some(parent) = full.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&full, value).await?;

        // Secrets — owner-only read/write.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            tokio::fs::set_permissions(&full, perms).await?;
        }

        tracing::debug!(path = %path, "secret store put: path={path}");
        Ok(())
    }

    async fn get(&self, path: &str) -> anyhow::Result<Option<Vec<u8>>> {
        validate_path(path)?;
        let full = self.full_path(path);
        match tokio::fs::read(&full).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, path: &str) -> anyhow::Result<()> {
        validate_path(path)?;
        let full = self.full_path(path);
        match tokio::fs::remove_file(&full).await {
            Ok(()) => {
                tracing::debug!(path = %path, "secret store delete: path={path}");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn list(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        if !prefix.is_empty() {
            validate_path(prefix)?;
        }
        self.walk_prefix(prefix).await
    }
}

/// A [`SecretStore`] that never stores anything — errors on every call.
///
/// Wired when `[secret_store]` is absent from the config. Lets the daemon
/// start and serve read-only traffic, while making it obvious at
/// operation time that nothing requiring secret storage can work.
#[derive(Debug, Default)]
pub struct NullSecretStore;

const NULL_MSG: &str = "no secret store configured — add a [secret_store] section to wardnet.toml to enable tunnels and backup";

#[async_trait]
impl SecretStore for NullSecretStore {
    async fn put(&self, _path: &str, _value: &[u8]) -> anyhow::Result<()> {
        anyhow::bail!(NULL_MSG)
    }

    async fn get(&self, _path: &str) -> anyhow::Result<Option<Vec<u8>>> {
        anyhow::bail!(NULL_MSG)
    }

    async fn delete(&self, _path: &str) -> anyhow::Result<()> {
        anyhow::bail!(NULL_MSG)
    }

    async fn list(&self, _prefix: &str) -> anyhow::Result<Vec<String>> {
        anyhow::bail!(NULL_MSG)
    }
}

/// Construct a [`SecretStore`] from the operator-supplied configuration.
///
/// Returns a [`NullSecretStore`] when `config` is `None` so callers stay
/// agnostic about whether a store was configured — operations fail with a
/// clear error at call time rather than at start-up.
///
/// Single factory wired by both `wardnetd` and `wardnetd-mock`; adding a
/// new provider variant only requires extending this match.
pub fn build_secret_store(config: Option<&SecretStoreConfig>) -> Arc<dyn SecretStore> {
    match config {
        Some(SecretStoreConfig::FileSystem { path }) => {
            tracing::info!(
                path = %path.display(),
                "secret store: file_system provider configured at path={path}",
                path = path.display(),
            );
            Arc::new(FileSecretStore::new(path.clone()))
        }
        None => {
            tracing::warn!(
                "secret store: no [secret_store] section in config — tunnels and backup will be unavailable"
            );
            Arc::new(NullSecretStore)
        }
    }
}
