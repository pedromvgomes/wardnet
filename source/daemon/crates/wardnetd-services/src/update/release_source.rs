//! Release discovery — abstraction over the manifest server and its assets.

use async_trait::async_trait;
use wardnet_common::update::{Release, UpdateChannel};

/// Resolves releases for a given channel and fetches their binary assets.
///
/// The real implementation fetches `<manifest_base_url>/<channel>.json` over
/// HTTPS and downloads tarball / digest / signature files from the URLs the
/// manifest resolves to. Mock and test implementations return preconfigured
/// bytes. Grouping the manifest lookup and asset fetch on one trait means
/// transport concerns stay in one place and the service layer stays
/// transport-agnostic.
#[async_trait]
pub trait ReleaseSource: Send + Sync {
    /// Return the latest release for the given channel, or `None` if the
    /// channel has no published release.
    async fn latest(&self, channel: UpdateChannel) -> anyhow::Result<Option<Release>>;

    /// Download an arbitrary asset URL (tarball, sha256 sidecar, minisig)
    /// as raw bytes.
    async fn fetch_asset(&self, url: &str) -> anyhow::Result<Vec<u8>>;
}
