//! SHA-256 + minisign verification backed by the embedded public key.

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::update::verifier::ReleaseVerifier;

/// Verifies tarballs against their SHA-256 sidecar and minisign signature.
///
/// The public verification key is provided by the caller (typically baked
/// into the binary at compile time via `include_str!`), so the only way to
/// install a new release is to produce a tarball signed by the matching
/// private key.
pub struct Sha256MinisignVerifier {
    public_key_text: &'static str,
}

impl Sha256MinisignVerifier {
    /// Construct a verifier bound to the given embedded public key.
    #[must_use]
    pub fn new(public_key_text: &'static str) -> Self {
        Self { public_key_text }
    }
}

#[async_trait]
impl ReleaseVerifier for Sha256MinisignVerifier {
    async fn verify_sha256(&self, tarball: &[u8], expected_hex: &str) -> anyhow::Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(tarball);
        let actual = hex::encode(hasher.finalize());
        let expected = expected_hex.trim().to_ascii_lowercase();
        if actual != expected {
            return Err(anyhow::anyhow!(
                "sha256 mismatch: expected={expected}, actual={actual}"
            ));
        }
        Ok(())
    }

    async fn verify_signature(&self, tarball: &[u8], signature: &[u8]) -> anyhow::Result<()> {
        let pk = minisign_verify::PublicKey::decode(self.public_key_text.trim())
            .map_err(|e| anyhow::anyhow!("invalid embedded public key: {e}"))?;
        let sig_text = std::str::from_utf8(signature)
            .map_err(|e| anyhow::anyhow!("signature is not utf-8: {e}"))?;
        let sig = minisign_verify::Signature::decode(sig_text)
            .map_err(|e| anyhow::anyhow!("invalid signature format: {e}"))?;
        pk.verify(tarball, &sig, /* allow_legacy */ false)
            .map_err(|e| anyhow::anyhow!("signature verification failed: {e}"))?;
        Ok(())
    }
}
