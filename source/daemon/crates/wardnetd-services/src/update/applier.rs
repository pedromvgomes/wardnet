//! Binary apply step — extract, stage, atomically swap.

use async_trait::async_trait;
use std::path::PathBuf;

/// Result of a successful swap — holds the path of the previous binary
/// retained for rollback.
#[derive(Debug, Clone)]
pub struct SwapOutcome {
    /// Where the previously-live binary was moved to (usually `<live>.old`).
    pub previous_binary: PathBuf,
}

/// Applies a verified release tarball to the running binary path.
///
/// The real implementation extracts the tarball into a staging directory,
/// renames the current live binary to `<live>.old`, then renames the
/// extracted binary into place. All three paths must live on the same
/// filesystem so the renames are atomic.
#[async_trait]
pub trait BinaryApplier: Send + Sync {
    /// Apply a verified tarball. On success, the running binary has been
    /// replaced and a `<live>.old` copy retained for rollback. Callers are
    /// expected to restart the daemon after this returns.
    async fn apply(&self, tarball: &[u8]) -> anyhow::Result<SwapOutcome>;

    /// Roll back to the `<live>.old` binary, if one exists. Returns an error
    /// if no rollback target is present.
    async fn rollback(&self) -> anyhow::Result<()>;

    /// Whether a `<live>.old` binary is present that can be rolled back to.
    async fn rollback_available(&self) -> bool;
}
