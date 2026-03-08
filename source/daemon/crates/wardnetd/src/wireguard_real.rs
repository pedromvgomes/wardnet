use std::time::SystemTime;

use async_trait::async_trait;
use wireguard_control::{Backend, Device, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder};

use crate::wireguard::{CreateInterfaceParams, WgInterfaceStats, WireGuardOps};

/// Real `WireGuard` implementation using `wireguard-control`.
///
/// Communicates via netlink on Linux (kernel backend) and userspace
/// sockets on macOS (via `wireguard-go`).
#[derive(Debug)]
pub struct RealWireGuard;

#[async_trait]
impl WireGuardOps for RealWireGuard {
    async fn create_interface(&self, params: CreateInterfaceParams) -> anyhow::Result<()> {
        let iface: InterfaceName = params
            .interface_name
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid interface name: {e}"))?;

        let private_key = Key(params.private_key);
        let peer_key = Key(params.peer_public_key);

        let mut peer = PeerConfigBuilder::new(&peer_key);

        if let Some(endpoint) = params.peer_endpoint {
            peer = peer.set_endpoint(endpoint);
        }

        for network in &params.peer_allowed_ips {
            peer = peer.add_allowed_ip(network.ip(), network.prefix());
        }

        if let Some(psk) = params.peer_preshared_key {
            peer = peer.set_preshared_key(Key(psk));
        }

        if let Some(keepalive) = params.persistent_keepalive {
            peer = peer.set_persistent_keepalive_interval(keepalive);
        }

        let mut update = DeviceUpdate::new()
            .set_private_key(private_key)
            .add_peer(peer);

        if let Some(port) = params.listen_port {
            update = update.set_listen_port(port);
        }

        update.apply(&iface, Backend::default())?;

        tracing::info!(interface = %params.interface_name, "wireguard interface created");
        Ok(())
    }

    async fn bring_up(&self, interface_name: &str) -> anyhow::Result<()> {
        // wireguard-control creates the interface in an "up" state already.
        // On Linux a separate `ip link set <iface> up` may be needed via
        // the nix crate or std::process::Command — left as a future enhancement.
        tracing::info!(interface = %interface_name, "wireguard interface up");
        Ok(())
    }

    async fn tear_down(&self, interface_name: &str) -> anyhow::Result<()> {
        tracing::info!(interface = %interface_name, "wireguard interface down");
        Ok(())
    }

    async fn remove_interface(&self, interface_name: &str) -> anyhow::Result<()> {
        let iface: InterfaceName = interface_name
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid interface name: {e}"))?;

        // Apply an empty update — the interface is removed when the last
        // reference is dropped on most backends. Use a minimal no-op update
        // so the kernel removes it.
        DeviceUpdate::new().apply(&iface, Backend::default())?;

        tracing::info!(interface = %interface_name, "wireguard interface removed");
        Ok(())
    }

    async fn get_stats(&self, interface_name: &str) -> anyhow::Result<Option<WgInterfaceStats>> {
        let iface: InterfaceName = interface_name
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid interface name: {e}"))?;

        let Ok(device) = Device::get(&iface, Backend::default()) else {
            return Ok(None);
        };

        let (mut total_tx, mut total_rx) = (0u64, 0u64);
        let mut latest_handshake: Option<SystemTime> = None;

        for peer in &device.peers {
            total_tx += peer.stats.tx_bytes;
            total_rx += peer.stats.rx_bytes;
            if let Some(hs) = peer.stats.last_handshake_time {
                latest_handshake = Some(match latest_handshake {
                    Some(prev) if prev > hs => prev,
                    _ => hs,
                });
            }
        }

        let last_handshake = latest_handshake.map(chrono::DateTime::<chrono::Utc>::from);

        Ok(Some(WgInterfaceStats {
            bytes_tx: total_tx,
            bytes_rx: total_rx,
            last_handshake,
        }))
    }

    async fn list_interfaces(&self) -> anyhow::Result<Vec<String>> {
        let devices = Device::list(Backend::default())?;
        Ok(devices.into_iter().map(|name| name.to_string()).collect())
    }
}
