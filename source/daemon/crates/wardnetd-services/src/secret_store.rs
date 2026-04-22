//! Secret storage (re-exported from `wardnetd-data`).
//!
//! Re-exported here so crates that depend on `wardnetd-services` (e.g.
//! `wardnetd-api`, `wardnetd`, `wardnetd-mock`) do not need a direct
//! dependency on `wardnetd-data`.
//!
//! Only [`SecretStore`], [`FileSecretStore`], [`NullSecretStore`], and
//! [`build_secret_store`] are re-exported — the narrower `KeyStore` trait
//! is intentionally kept private to the tunnel service, so nothing outside
//! `wardnetd-services::tunnel` depends on it.
pub use wardnetd_data::secret_store::{
    FileSecretStore, NullSecretStore, SecretStore, build_secret_store,
};
