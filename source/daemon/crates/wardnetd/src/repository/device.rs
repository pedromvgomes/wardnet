use async_trait::async_trait;
use wardnet_types::device::Device;
use wardnet_types::routing::RoutingRule;

/// Data access for devices and their routing rules.
///
/// Provides lookups by IP and ID, routing rule queries, and upserts.
/// All business logic (e.g. admin-lock checks) belongs in
/// [`DeviceService`](crate::service::DeviceService).
#[async_trait]
pub trait DeviceRepository: Send + Sync {
    /// Find a device by its most recently observed IP address.
    async fn find_by_ip(&self, ip: &str) -> anyhow::Result<Option<Device>>;

    /// Find a device by its primary key.
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Device>>;

    /// Return the routing rule for a device, if one exists.
    async fn find_rule_for_device(&self, device_id: &str) -> anyhow::Result<Option<RoutingRule>>;

    /// Insert or update a user-created routing rule for a device.
    async fn upsert_user_rule(
        &self,
        device_id: &str,
        target_json: &str,
        now: &str,
    ) -> anyhow::Result<()>;

    /// Return the total number of devices.
    async fn count(&self) -> anyhow::Result<i64>;
}
