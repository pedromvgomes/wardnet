use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use wardnetd::config::{Config, LogFormat};
use wardnetd::event::{BroadcastEventBus, EventPublisher};
use wardnetd::keys::FileKeyStore;
use wardnetd::repository::{
    SqliteAdminRepository, SqliteApiKeyRepository, SqliteDeviceRepository, SqliteSessionRepository,
    SqliteSystemConfigRepository, SqliteTunnelRepository,
};
use wardnetd::service::{AuthServiceImpl, DeviceServiceImpl, SystemServiceImpl, TunnelServiceImpl};
use wardnetd::state::AppState;
use wardnetd::tunnel_idle::IdleTunnelWatcher;
use wardnetd::tunnel_monitor::TunnelMonitor;
use wardnetd::wireguard::WireGuardOps;
use wardnetd::wireguard_noop::NoopWireGuard;
use wardnetd::wireguard_real::RealWireGuard;
use wardnetd::{api, db};

/// Wardnet daemon — self-hosted network privacy gateway.
#[derive(Parser)]
#[command(name = "wardnetd", version)]
struct Cli {
    /// Path to configuration file.
    #[arg(long, short, default_value = "/etc/wardnet/wardnet.toml")]
    config: PathBuf,

    /// Enable verbose (debug-level) logging. Overrides the config file log level.
    #[arg(long, short)]
    verbose: bool,

    /// Run in foreground mode (default). Use with systemd or as a regular process.
    #[arg(long)]
    foreground: bool,

    /// Use a no-op `WireGuard` backend instead of real kernel interfaces.
    #[arg(long)]
    mock_network: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;

    init_tracing(&config, cli.verbose);

    let started_at = Instant::now();

    let pool = db::init_pool(&config.database.path).await?;

    // Create repository instances (concrete SQLite implementations).
    let admin_repo = Arc::new(SqliteAdminRepository::new(pool.clone()));
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let api_key_repo = Arc::new(SqliteApiKeyRepository::new(pool.clone()));
    let device_repo = Arc::new(SqliteDeviceRepository::new(pool.clone()));
    let system_config_repo = Arc::new(SqliteSystemConfigRepository::new(pool.clone()));
    let tunnel_repo = Arc::new(SqliteTunnelRepository::new(pool));

    // Create event publisher.
    let event_publisher: Arc<dyn EventPublisher> = Arc::new(BroadcastEventBus::new(256));

    // Select WireGuard backend.
    let wireguard: Arc<dyn WireGuardOps> = if cli.mock_network {
        tracing::info!("using no-op WireGuard backend (--mock-network)");
        Arc::new(NoopWireGuard)
    } else {
        Arc::new(RealWireGuard)
    };

    // Create key store.
    let key_store = Arc::new(FileKeyStore::new(config.tunnel.keys_dir.clone()));

    // Create service instances, injecting repository traits.
    let auth_service = Arc::new(AuthServiceImpl::new(
        admin_repo,
        session_repo,
        api_key_repo,
        config.auth.session_expiry_hours,
    ));
    let device_service = Arc::new(DeviceServiceImpl::new(device_repo));
    let system_service = Arc::new(SystemServiceImpl::new(system_config_repo, started_at));
    let tunnel_service: Arc<dyn wardnetd::service::TunnelService> =
        Arc::new(TunnelServiceImpl::new(
            tunnel_repo.clone(),
            wireguard.clone(),
            key_store,
            event_publisher.clone(),
        ));

    // Restore tunnel configurations from the database.
    tunnel_service
        .restore_tunnels()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Start background monitors.
    let monitor = TunnelMonitor::start(
        tunnel_repo,
        wireguard,
        event_publisher.clone(),
        config.tunnel.stats_interval_secs,
        config.tunnel.health_check_interval_secs,
    );
    let idle_watcher = IdleTunnelWatcher::start(
        event_publisher.clone(),
        tunnel_service.clone(),
        config.tunnel.idle_timeout_secs,
    );

    let state = AppState::new(
        auth_service,
        device_service,
        system_service,
        tunnel_service,
        event_publisher,
        config.clone(),
    );

    let app = api::router(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    println!(
        "\n  Wardnet daemon v{}\n  Listening on http://{}\n  Database: {}\n",
        env!("CARGO_PKG_VERSION"),
        addr,
        config.database.path.display()
    );

    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("server stopped, shutting down background tasks");
    monitor.shutdown().await;
    idle_watcher.shutdown().await;

    Ok(())
}

/// Initialize the tracing subscriber. When `verbose` is true, the log level
/// is forced to `debug` regardless of config or environment.
fn init_tracing(config: &Config, verbose: bool) {
    let level = if verbose { "debug" } else { "error" };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let subscriber = tracing_subscriber::fmt().with_env_filter(filter);

    match config.logging.format {
        LogFormat::Json => subscriber.json().init(),
        LogFormat::Console => subscriber.init(),
    }
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
    }

    tracing::info!("shutdown signal received");
}
