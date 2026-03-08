use std::net::SocketAddr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;

/// Parameters for creating a `WireGuard` interface.
#[derive(Debug, Clone)]
pub struct CreateInterfaceParams {
    /// Name of the `WireGuard` interface (e.g. "wg0").
    pub interface_name: String,
    /// Raw 32-byte private key.
    pub private_key: [u8; 32],
    /// Optional listen port for incoming connections.
    pub listen_port: Option<u16>,
    /// Raw 32-byte peer public key.
    pub peer_public_key: [u8; 32],
    /// Remote endpoint of the peer.
    pub peer_endpoint: Option<SocketAddr>,
    /// CIDRs the peer is allowed to route.
    pub peer_allowed_ips: Vec<IpNetwork>,
    /// Optional raw 32-byte preshared key.
    pub peer_preshared_key: Option<[u8; 32]>,
    /// Keepalive interval in seconds, if desired.
    pub persistent_keepalive: Option<u16>,
}

/// Live stats from a `WireGuard` interface.
#[derive(Debug, Clone)]
pub struct WgInterfaceStats {
    /// Total bytes transmitted across all peers.
    pub bytes_tx: u64,
    /// Total bytes received across all peers.
    pub bytes_rx: u64,
    /// Most recent handshake time among all peers.
    pub last_handshake: Option<DateTime<Utc>>,
}

/// Abstraction over `WireGuard` kernel/userspace operations.
///
/// Enables mocking in tests. The real implementation uses `wireguard-control`
/// which communicates via netlink on Linux and userspace sockets on macOS.
/// Requires `CAP_NET_ADMIN` capability on Linux.
#[async_trait]
pub trait WireGuardOps: Send + Sync {
    /// Create a `WireGuard` interface and configure it with a peer.
    async fn create_interface(&self, params: CreateInterfaceParams) -> anyhow::Result<()>;

    /// Bring a `WireGuard` interface up (set link state to up).
    async fn bring_up(&self, interface_name: &str) -> anyhow::Result<()>;

    /// Tear down a `WireGuard` interface (set link state to down).
    async fn tear_down(&self, interface_name: &str) -> anyhow::Result<()>;

    /// Remove a `WireGuard` interface entirely.
    async fn remove_interface(&self, interface_name: &str) -> anyhow::Result<()>;

    /// Query the current stats for a `WireGuard` interface.
    async fn get_stats(&self, interface_name: &str) -> anyhow::Result<Option<WgInterfaceStats>>;

    /// List all existing `WireGuard` interface names.
    async fn list_interfaces(&self) -> anyhow::Result<Vec<String>>;
}
