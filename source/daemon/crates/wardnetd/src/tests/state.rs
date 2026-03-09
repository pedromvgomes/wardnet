use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tokio::sync::broadcast;
use uuid::Uuid;
use wardnet_types::api::{
    CreateTunnelRequest, CreateTunnelResponse, DeleteTunnelResponse, DeviceMeResponse,
    ListProvidersResponse, ListServersRequest, ListServersResponse, ListTunnelsResponse,
    SetMyRuleResponse, SetupProviderRequest, SetupProviderResponse, SystemStatusResponse,
    ValidateCredentialsRequest, ValidateCredentialsResponse,
};
use wardnet_types::device::{Device, DeviceType};
use wardnet_types::event::WardnetEvent;
use wardnet_types::routing::RoutingTarget;
use wardnet_types::tunnel::Tunnel;

use crate::config::Config;
use crate::error::AppError;
use crate::event::EventPublisher;
use crate::packet_capture::ObservedDevice;
use crate::service::auth::LoginResult;
use crate::service::discovery::ObservationResult;
use crate::service::{
    AuthService, DeviceDiscoveryService, DeviceService, ProviderService, SystemService,
    TunnelService,
};
use crate::state::AppState;

// -- Stub services for constructing AppState ---------------------------------

struct StubAuth;

#[async_trait]
impl AuthService for StubAuth {
    async fn login(&self, _u: &str, _p: &str) -> Result<LoginResult, AppError> {
        unimplemented!()
    }
    async fn validate_session(&self, _t: &str) -> Result<Option<Uuid>, AppError> {
        Ok(None)
    }
    async fn validate_api_key(&self, _k: &str) -> Result<Option<Uuid>, AppError> {
        Ok(None)
    }
    async fn setup_admin(&self, _u: &str, _p: &str) -> Result<(), AppError> {
        Ok(())
    }
    async fn is_setup_completed(&self) -> Result<bool, AppError> {
        Ok(false)
    }
}

struct StubDevice;

#[async_trait]
impl DeviceService for StubDevice {
    async fn get_device_for_ip(&self, _ip: &str) -> Result<DeviceMeResponse, AppError> {
        unimplemented!()
    }
    async fn set_rule_for_ip(
        &self,
        _ip: &str,
        _target: RoutingTarget,
    ) -> Result<SetMyRuleResponse, AppError> {
        unimplemented!()
    }
}

struct StubDiscovery;

#[async_trait]
impl DeviceDiscoveryService for StubDiscovery {
    async fn restore_devices(&self) -> Result<(), AppError> {
        Ok(())
    }
    async fn process_observation(
        &self,
        _obs: &ObservedDevice,
    ) -> Result<ObservationResult, AppError> {
        unimplemented!()
    }
    async fn flush_last_seen(&self) -> Result<u64, AppError> {
        Ok(0)
    }
    async fn scan_departures(&self, _timeout_secs: u64) -> Result<Vec<Uuid>, AppError> {
        Ok(vec![])
    }
    async fn resolve_hostname(&self, _id: Uuid, _ip: String) -> Result<(), AppError> {
        Ok(())
    }
    async fn get_all_devices(&self) -> Result<Vec<Device>, AppError> {
        Ok(vec![])
    }
    async fn get_device_by_id(&self, _id: Uuid) -> Result<Device, AppError> {
        unimplemented!()
    }
    async fn update_device(
        &self,
        _id: Uuid,
        _name: Option<&str>,
        _device_type: Option<DeviceType>,
    ) -> Result<Device, AppError> {
        unimplemented!()
    }
}

struct StubProvider;

#[async_trait]
impl ProviderService for StubProvider {
    async fn list_providers(&self) -> Result<ListProvidersResponse, AppError> {
        Ok(ListProvidersResponse { providers: vec![] })
    }
    async fn validate_credentials(
        &self,
        _id: &str,
        _req: ValidateCredentialsRequest,
    ) -> Result<ValidateCredentialsResponse, AppError> {
        unimplemented!()
    }
    async fn list_servers(
        &self,
        _id: &str,
        _req: ListServersRequest,
    ) -> Result<ListServersResponse, AppError> {
        unimplemented!()
    }
    async fn setup_tunnel(
        &self,
        _id: &str,
        _req: SetupProviderRequest,
    ) -> Result<SetupProviderResponse, AppError> {
        unimplemented!()
    }
}

struct StubSystem;

#[async_trait]
impl SystemService for StubSystem {
    async fn status(&self) -> Result<SystemStatusResponse, AppError> {
        unimplemented!()
    }
}

struct StubTunnel;

#[async_trait]
impl TunnelService for StubTunnel {
    async fn import_tunnel(
        &self,
        _req: CreateTunnelRequest,
    ) -> Result<CreateTunnelResponse, AppError> {
        unimplemented!()
    }
    async fn list_tunnels(&self) -> Result<ListTunnelsResponse, AppError> {
        Ok(ListTunnelsResponse { tunnels: vec![] })
    }
    async fn get_tunnel(&self, _id: Uuid) -> Result<Tunnel, AppError> {
        unimplemented!()
    }
    async fn bring_up(&self, _id: Uuid) -> Result<(), AppError> {
        Ok(())
    }
    async fn tear_down(&self, _id: Uuid, _reason: &str) -> Result<(), AppError> {
        Ok(())
    }
    async fn delete_tunnel(&self, _id: Uuid) -> Result<DeleteTunnelResponse, AppError> {
        unimplemented!()
    }
    async fn restore_tunnels(&self) -> Result<(), AppError> {
        Ok(())
    }
}

struct StubEvents;

impl EventPublisher for StubEvents {
    fn publish(&self, _event: WardnetEvent) {}
    fn subscribe(&self) -> broadcast::Receiver<WardnetEvent> {
        let (_, rx) = broadcast::channel(1);
        rx
    }
}

fn build_state() -> AppState {
    AppState::new(
        Arc::new(StubAuth),
        Arc::new(StubDevice),
        Arc::new(StubDiscovery),
        Arc::new(StubProvider),
        Arc::new(StubSystem),
        Arc::new(StubTunnel),
        Arc::new(StubEvents),
        Config::default(),
        Instant::now(),
    )
}

#[test]
fn accessors_return_correct_types() {
    let state = build_state();

    // Exercise every accessor to confirm they compile and run without panic.
    let _ = state.auth_service();
    let _ = state.device_service();
    let _ = state.discovery_service();
    let _ = state.provider_service();
    let _ = state.system_service();
    let _ = state.tunnel_service();
    let _ = state.event_publisher();
    let _ = state.config();
    let _ = state.started_at();
}

#[test]
fn clone_shares_inner_state() {
    let state = build_state();
    let cloned = state.clone();

    // Both should report the same started_at instant.
    assert_eq!(state.started_at(), cloned.started_at());
    assert_eq!(state.config().server.port, cloned.config().server.port);
}
