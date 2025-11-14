// Integration tests for webrtc_peer

use webrtc_peer::*;

#[tokio::test]
async fn test_full_peer_connection_workflow() {
    // This test demonstrates a complete WebRTC peer connection workflow

    // Create configuration
    let config = RtcConfiguration {
        ice_servers: vec![IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        }],
        ice_transport_policy: IceTransportPolicy::All,
        bundle_policy: BundlePolicy::Balanced,
    };

    // Create peer connection
    let peer = RtcPeerConnection::new(config).await;
    assert!(peer.is_ok());

    let peer = peer.unwrap();
    assert!(!peer.connection_id.to_string().is_empty());
}

#[tokio::test]
async fn test_multiple_peer_connections() {
    // Test creating multiple peer connections simultaneously

    let config = RtcConfiguration {
        ice_servers: vec![IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        }],
        ice_transport_policy: IceTransportPolicy::All,
        bundle_policy: BundlePolicy::Balanced,
    };

    let peer1 = RtcPeerConnection::new(config.clone()).await.unwrap();
    let peer2 = RtcPeerConnection::new(config).await.unwrap();

    // Ensure they have different connection IDs
    assert_ne!(peer1.connection_id, peer2.connection_id);
}

#[tokio::test]
async fn test_peer_connection_with_turn_server() {
    // Test configuration with TURN server credentials

    let config = RtcConfiguration {
        ice_servers: vec![
            IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            },
            IceServer {
                urls: vec!["turn:turn.example.com:3478".to_string()],
                username: Some("testuser".to_string()),
                credential: Some("testpass".to_string()),
            },
        ],
        ice_transport_policy: IceTransportPolicy::All,
        bundle_policy: BundlePolicy::Balanced,
    };

    let peer = RtcPeerConnection::new(config).await;
    assert!(peer.is_ok());
}
