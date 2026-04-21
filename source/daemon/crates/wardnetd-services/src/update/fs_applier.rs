//! Filesystem [`BinaryApplier`] — extract, stage, rename-swap.

use std::io::Read;
use std::path::PathBuf;

use async_trait::async_trait;
use flate2::read::GzDecoder;
use tokio::task;

use crate::update::applier::{BinaryApplier, SwapOutcome};

/// Write-then-rename applier bound to a live binary path and staging dir.
///
/// The live binary and the staging directory must live on the same
/// filesystem so the final `rename(2)` is atomic. `apply()` leaves the
/// previous binary at `<live>.old`, which is the target `rollback()`
/// swaps back into place.
pub struct FsBinaryApplier {
    live_path: PathBuf,
    staging_dir: PathBuf,
}

impl FsBinaryApplier {
    /// Construct a new applier.
    #[must_use]
    pub fn new(live_path: PathBuf, staging_dir: PathBuf) -> Self {
        Self {
            live_path,
            staging_dir,
        }
    }

    fn old_path(&self) -> PathBuf {
        let mut p = self.live_path.clone();
        let name = p.file_name().map_or_else(
            || "wardnetd.old".to_owned(),
            |n| format!("{}.old", n.to_string_lossy()),
        );
        p.set_file_name(name);
        p
    }

    fn extract_and_swap(&self, tarball: &[u8]) -> anyhow::Result<SwapOutcome> {
        std::fs::create_dir_all(&self.staging_dir)?;

        let decoder = GzDecoder::new(tarball);
        let mut archive = tar::Archive::new(decoder);

        let mut staged_binary: Option<PathBuf> = None;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if name != "wardnetd" {
                continue;
            }
            let staged = self.staging_dir.join("wardnetd.staged");
            let mut out = std::fs::File::create(&staged)?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            std::io::Write::write_all(&mut out, &buf)?;
            drop(out);
            // Ensure the staged binary is executable.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&staged)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&staged, perms)?;
            }
            staged_binary = Some(staged);
            break;
        }

        let staged = staged_binary.ok_or_else(|| {
            anyhow::anyhow!("tarball did not contain a `wardnetd` entry at any path")
        })?;

        let old_path = self.old_path();
        // Move current live binary aside. If no live binary exists (first install
        // on a fresh host), skip this step — rollback simply won't be available
        // until the next upgrade.
        if self.live_path.exists() {
            if old_path.exists() {
                std::fs::remove_file(&old_path)?;
            }
            std::fs::rename(&self.live_path, &old_path)?;
        }

        // Atomic rename into the live path.
        std::fs::rename(&staged, &self.live_path)?;

        Ok(SwapOutcome {
            previous_binary: old_path,
        })
    }
}

#[async_trait]
impl BinaryApplier for FsBinaryApplier {
    async fn apply(&self, tarball: &[u8]) -> anyhow::Result<SwapOutcome> {
        let tarball = tarball.to_vec();
        let this_live = self.live_path.clone();
        let this_staging = self.staging_dir.clone();
        task::spawn_blocking(move || {
            let applier = FsBinaryApplier::new(this_live, this_staging);
            applier.extract_and_swap(&tarball)
        })
        .await?
    }

    async fn rollback(&self) -> anyhow::Result<()> {
        let old = self.old_path();
        if !old.exists() {
            return Err(anyhow::anyhow!(
                "no previous binary at {}",
                old.to_string_lossy()
            ));
        }
        let live = self.live_path.clone();
        task::spawn_blocking(move || -> anyhow::Result<()> {
            if live.exists() {
                std::fs::remove_file(&live)?;
            }
            std::fs::rename(&old, &live)?;
            Ok(())
        })
        .await?
    }

    async fn rollback_available(&self) -> bool {
        self.old_path().exists()
    }
}
