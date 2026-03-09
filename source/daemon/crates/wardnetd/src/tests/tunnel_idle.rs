use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use wardnet_types::api::{
    CreateTunnelRequest, CreateTunnelResponse, DeleteTunnelResponse, ListTunnelsResponse,
};
use wardnet_types::event::WardnetEvent;
use wardnet_types::tunnel::Tunnel;

use crate::error::AppError;
use crate::event::{BroadcastEventBus, EventPublisher};
use crate::service::TunnelService;
use crate::tunnel_idle::IdleTunnelWatcher;

// -- Mock TunnelService ---------------------------------------------------

/// Tracks calls to `tear_down` for assertion.
struct MockTunnelService {
    tear_downs: Mutex<Vec<(Uuid, String)>>,
}

impl MockTunnelService {
    fn new() -> Self {
        Self {
            tear_downs: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl TunnelService for MockTunnelService {
    async fn import_tunnel(
        &self,
        _req: CreateTunnelRequest,
    ) -> Result<CreateTunnelResponse, AppError> {
        unimplemented!("not needed for idle watcher tests")
    }

    async fn list_tunnels(&self) -> Result<ListTunnelsResponse, AppError> {
        Ok(ListTunnelsResponse {
            tunnels: Vec::new(),
        })
    }

    async fn get_tunnel(&self, _id: Uuid) -> Result<Tunnel, AppError> {
        Err(AppError::NotFound("not found".to_owned()))
    }

    async fn bring_up(&self, _id: Uuid) -> Result<(), AppError> {
        Ok(())
    }

    async fn tear_down(&self, id: Uuid, reason: &str) -> Result<(), AppError> {
        self.tear_downs
            .lock()
            .unwrap()
            .push((id, reason.to_owned()));
        Ok(())
    }

    async fn delete_tunnel(&self, _id: Uuid) -> Result<DeleteTunnelResponse, AppError> {
        unimplemented!("not needed for idle watcher tests")
    }

    async fn restore_tunnels(&self) -> Result<(), AppError> {
        Ok(())
    }
}

// -- Tests ----------------------------------------------------------------

#[tokio::test]
async fn watcher_receives_events_from_bus() {
    let bus = Arc::new(BroadcastEventBus::new(16));
    let tunnel_svc = Arc::new(MockTunnelService::new());

    let parent = tracing::info_span!("test");
    let watcher = IdleTunnelWatcher::start(bus.clone(), tunnel_svc, 300, &parent);

    // Publish a DeviceGone event.
    bus.publish(WardnetEvent::DeviceGone {
        device_id: Uuid::new_v4(),
        mac: "AA:BB:CC:DD:EE:01".to_owned(),
        last_ip: "192.168.1.10".to_owned(),
        timestamp: Utc::now(),
    });

    // Give the event loop time to process.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    watcher.shutdown().await;
    // The current stub just logs events; if we reach here it processed without panic.
}

#[tokio::test]
async fn watcher_handles_multiple_events() {
    let bus = Arc::new(BroadcastEventBus::new(16));
    let tunnel_svc = Arc::new(MockTunnelService::new());

    let parent = tracing::info_span!("test");
    let watcher = IdleTunnelWatcher::start(bus.clone(), tunnel_svc, 300, &parent);

    // Publish several events in sequence.
    for i in 0..5 {
        bus.publish(WardnetEvent::DeviceGone {
            device_id: Uuid::new_v4(),
            mac: format!("AA:BB:CC:DD:EE:{i:02X}"),
            last_ip: format!("192.168.1.{}", 10 + i),
            timestamp: Utc::now(),
        });
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    watcher.shutdown().await;
    // All events should be consumed without panic.
}

#[tokio::test]
async fn watcher_handles_tunnel_events() {
    let bus = Arc::new(BroadcastEventBus::new(16));
    let tunnel_svc = Arc::new(MockTunnelService::new());

    let parent = tracing::info_span!("test");
    let watcher = IdleTunnelWatcher::start(bus.clone(), tunnel_svc, 300, &parent);

    bus.publish(WardnetEvent::TunnelUp {
        tunnel_id: Uuid::new_v4(),
        interface_name: "wg_ward0".to_owned(),
        endpoint: "198.51.100.1:51820".to_owned(),
        timestamp: Utc::now(),
    });

    bus.publish(WardnetEvent::TunnelDown {
        tunnel_id: Uuid::new_v4(),
        interface_name: "wg_ward0".to_owned(),
        reason: "idle".to_owned(),
        timestamp: Utc::now(),
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    watcher.shutdown().await;
}

#[tokio::test]
async fn watcher_shutdown_without_events() {
    let bus = Arc::new(BroadcastEventBus::new(16));
    let tunnel_svc = Arc::new(MockTunnelService::new());

    let parent = tracing::info_span!("test");
    let watcher = IdleTunnelWatcher::start(bus, tunnel_svc, 300, &parent);

    // Shutdown immediately with no events -- should complete cleanly.
    watcher.shutdown().await;
}

#[tokio::test]
async fn watcher_handles_device_discovered_event() {
    let bus = Arc::new(BroadcastEventBus::new(16));
    let tunnel_svc = Arc::new(MockTunnelService::new());

    let parent = tracing::info_span!("test");
    let watcher = IdleTunnelWatcher::start(bus.clone(), tunnel_svc, 300, &parent);

    bus.publish(WardnetEvent::DeviceDiscovered {
        device_id: Uuid::new_v4(),
        mac: "AA:BB:CC:DD:EE:01".to_owned(),
        ip: "192.168.1.10".to_owned(),
        hostname: Some("myphone".to_owned()),
        timestamp: Utc::now(),
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    watcher.shutdown().await;
}
