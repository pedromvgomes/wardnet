//! Release artefact verification — SHA-256 digest + minisign signature.

use async_trait::async_trait;

/// Verifies a downloaded release tarball against its expected digest and
/// signature.
///
/// The SHA-256 digest is always verified; signature verification is optional
/// and controlled by the daemon's [`UpdateConfig::require_signature`] flag
/// (`true` for production builds). Both operations take the tarball as
/// in-memory bytes so tests can exercise them without disk I/O.
#[async_trait]
pub trait ReleaseVerifier: Send + Sync {
    /// Verify the SHA-256 digest of the tarball against the expected value.
    async fn verify_sha256(&self, tarball: &[u8], expected_hex: &str) -> anyhow::Result<()>;

    /// Verify the minisign signature of the tarball using the embedded
    /// public key. Returns `Ok(())` on success, an error otherwise.
    async fn verify_signature(&self, tarball: &[u8], signature: &[u8]) -> anyhow::Result<()>;
}
