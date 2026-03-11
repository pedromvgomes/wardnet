use crate::tunnel_interface_wireguard::{PeerStatsInput, aggregate_peer_stats};
use std::time::{Duration, SystemTime};

#[test]
fn aggregate_empty_peers() {
    let stats = aggregate_peer_stats(&[]);

    assert_eq!(stats.bytes_tx, 0);
    assert_eq!(stats.bytes_rx, 0);
    assert!(stats.last_handshake.is_none());
}

#[test]
fn aggregate_single_peer() {
    let handshake = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
    let peers = vec![PeerStatsInput {
        tx_bytes: 100,
        rx_bytes: 200,
        last_handshake_time: Some(handshake),
    }];

    let stats = aggregate_peer_stats(&peers);

    assert_eq!(stats.bytes_tx, 100);
    assert_eq!(stats.bytes_rx, 200);
    assert_eq!(
        stats.last_handshake,
        Some(chrono::DateTime::<chrono::Utc>::from(handshake))
    );
}

#[test]
fn aggregate_multiple_peers_sums_bytes() {
    let peers = vec![
        PeerStatsInput {
            tx_bytes: 100,
            rx_bytes: 200,
            last_handshake_time: None,
        },
        PeerStatsInput {
            tx_bytes: 300,
            rx_bytes: 400,
            last_handshake_time: None,
        },
    ];

    let stats = aggregate_peer_stats(&peers);

    assert_eq!(stats.bytes_tx, 400);
    assert_eq!(stats.bytes_rx, 600);
}

#[test]
fn aggregate_picks_latest_handshake() {
    let earlier = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
    let later = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);

    let peers = vec![
        PeerStatsInput {
            tx_bytes: 0,
            rx_bytes: 0,
            last_handshake_time: Some(earlier),
        },
        PeerStatsInput {
            tx_bytes: 0,
            rx_bytes: 0,
            last_handshake_time: Some(later),
        },
    ];

    let stats = aggregate_peer_stats(&peers);

    assert_eq!(
        stats.last_handshake,
        Some(chrono::DateTime::<chrono::Utc>::from(later))
    );
}

#[test]
fn aggregate_ignores_none_handshakes() {
    let handshake = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);

    let peers = vec![
        PeerStatsInput {
            tx_bytes: 0,
            rx_bytes: 0,
            last_handshake_time: Some(handshake),
        },
        PeerStatsInput {
            tx_bytes: 0,
            rx_bytes: 0,
            last_handshake_time: None,
        },
    ];

    let stats = aggregate_peer_stats(&peers);

    assert_eq!(
        stats.last_handshake,
        Some(chrono::DateTime::<chrono::Utc>::from(handshake))
    );
}

#[test]
fn aggregate_all_none_handshakes() {
    let peers = vec![
        PeerStatsInput {
            tx_bytes: 10,
            rx_bytes: 20,
            last_handshake_time: None,
        },
        PeerStatsInput {
            tx_bytes: 30,
            rx_bytes: 40,
            last_handshake_time: None,
        },
    ];

    let stats = aggregate_peer_stats(&peers);

    assert_eq!(stats.bytes_tx, 40);
    assert_eq!(stats.bytes_rx, 60);
    assert!(stats.last_handshake.is_none());
}
