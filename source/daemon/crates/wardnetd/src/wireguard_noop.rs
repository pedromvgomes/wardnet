use async_trait::async_trait;

use crate::wireguard::{CreateInterfaceParams, WgInterfaceStats, WireGuardOps};

/// No-op `WireGuard` implementation for development and testing.
///
/// Logs all operations via `tracing::info!` without touching the kernel.
/// Used when running with `--mock-network`.
#[derive(Debug)]
pub struct NoopWireGuard;

#[async_trait]
impl WireGuardOps for NoopWireGuard {
    async fn create_interface(&self, params: CreateInterfaceParams) -> anyhow::Result<()> {
        tracing::info!(interface = %params.interface_name, "noop: create interface");
        Ok(())
    }

    async fn bring_up(&self, interface_name: &str) -> anyhow::Result<()> {
        tracing::info!(interface = %interface_name, "noop: bring up");
        Ok(())
    }

    async fn tear_down(&self, interface_name: &str) -> anyhow::Result<()> {
        tracing::info!(interface = %interface_name, "noop: tear down");
        Ok(())
    }

    async fn remove_interface(&self, interface_name: &str) -> anyhow::Result<()> {
        tracing::info!(interface = %interface_name, "noop: remove interface");
        Ok(())
    }

    async fn get_stats(&self, interface_name: &str) -> anyhow::Result<Option<WgInterfaceStats>> {
        tracing::info!(interface = %interface_name, "noop: get stats");
        Ok(None)
    }

    async fn list_interfaces(&self) -> anyhow::Result<Vec<String>> {
        tracing::info!("noop: list interfaces");
        Ok(vec![])
    }
}
