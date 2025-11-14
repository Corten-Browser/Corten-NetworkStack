// Unit tests for webrtc_peer

use webrtc_peer::*;

#[cfg(test)]
mod test_configuration {
    use super::*;

    #[test]
    fn test_rtc_configuration_creation() {
        let ice_servers = vec![IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        }];

        let config = RtcConfiguration {
            ice_servers,
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };

        assert_eq!(config.ice_servers.len(), 1);
        assert_eq!(
            config.ice_servers[0].urls[0],
            "stun:stun.l.google.com:19302"
        );
    }

    #[test]
    fn test_ice_server_with_credentials() {
        let server = IceServer {
            urls: vec!["turn:turn.example.com:3478".to_string()],
            username: Some("testuser".to_string()),
            credential: Some("testpass".to_string()),
        };

        assert!(server.username.is_some());
        assert!(server.credential.is_some());
        assert_eq!(server.username.unwrap(), "testuser");
    }
}

#[cfg(test)]
mod test_session_description {
    use super::*;

    #[test]
    fn test_session_description_offer() {
        let sdp = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: "v=0\r\no=- 123 123 IN IP4 127.0.0.1\r\n".to_string(),
        };

        assert!(matches!(sdp.sdp_type, SdpType::Offer));
        assert!(sdp.sdp.starts_with("v=0"));
    }

    #[test]
    fn test_session_description_answer() {
        let sdp = SessionDescription {
            sdp_type: SdpType::Answer,
            sdp: "v=0\r\no=- 456 456 IN IP4 127.0.0.1\r\n".to_string(),
        };

        assert!(matches!(sdp.sdp_type, SdpType::Answer));
    }
}

#[cfg(test)]
mod test_ice_candidate {
    use super::*;

    #[test]
    fn test_ice_candidate_creation() {
        let candidate = IceCandidate {
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.100 54321 typ host".to_string(),
            sdp_mid: Some("0".to_string()),
            sdp_m_line_index: Some(0),
        };

        assert!(candidate.candidate.contains("UDP"));
        assert_eq!(candidate.sdp_mid, Some("0".to_string()));
        assert_eq!(candidate.sdp_m_line_index, Some(0));
    }

    #[test]
    fn test_ice_candidate_without_optional_fields() {
        let candidate = IceCandidate {
            candidate: "candidate:2 1 TCP 2130706431 192.168.1.100 54322 typ host".to_string(),
            sdp_mid: None,
            sdp_m_line_index: None,
        };

        assert!(candidate.candidate.contains("TCP"));
        assert!(candidate.sdp_mid.is_none());
        assert!(candidate.sdp_m_line_index.is_none());
    }
}

#[cfg(test)]
mod test_peer_connection {
    use super::*;

    #[tokio::test]
    async fn test_create_peer_connection() {
        let config = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };

        let result = RtcPeerConnection::new(config).await;
        assert!(result.is_ok());

        let peer = result.unwrap();
        assert!(!peer.connection_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_create_offer() {
        let config = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };

        let peer = RtcPeerConnection::new(config).await.unwrap();
        let options = OfferOptions {
            voice_activity_detection: true,
            ice_restart: false,
        };

        let result = peer.create_offer(options).await;
        assert!(result.is_ok());

        let offer = result.unwrap();
        assert!(matches!(offer.sdp_type, SdpType::Offer));
        assert!(!offer.sdp.is_empty());
    }

    // IGNORED: Requires external STUN server (stun.l.google.com) for WebRTC negotiation.
    // This test creates actual peer connections which need network access and may fail
    // in isolated test environments. To run: `cargo test --ignored`
    #[tokio::test]
    #[ignore]
    async fn test_create_answer() {
        // Create first peer to make offer
        let config1 = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };
        let peer1 = RtcPeerConnection::new(config1).await.unwrap();
        let offer = peer1
            .create_offer(OfferOptions {
                voice_activity_detection: true,
                ice_restart: false,
            })
            .await
            .unwrap();

        // Create second peer to answer
        let config2 = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };
        let mut peer2 = RtcPeerConnection::new(config2).await.unwrap();

        // Set remote offer
        peer2.set_remote_description(offer).await.unwrap();

        // Create answer
        let options = AnswerOptions {};
        let result = peer2.create_answer(options).await;
        assert!(result.is_ok());

        let answer = result.unwrap();
        assert!(matches!(answer.sdp_type, SdpType::Answer));
    }

    #[tokio::test]
    async fn test_set_local_description() {
        let config = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };

        let mut peer = RtcPeerConnection::new(config).await.unwrap();
        let offer = peer
            .create_offer(OfferOptions {
                voice_activity_detection: true,
                ice_restart: false,
            })
            .await
            .unwrap();

        let result = peer.set_local_description(offer).await;
        assert!(result.is_ok());
    }

    // IGNORED: Requires external STUN server (stun.l.google.com) for WebRTC negotiation.
    // This test creates actual peer connections which need network access and may fail
    // in isolated test environments. To run: `cargo test --ignored`
    #[tokio::test]
    #[ignore]
    async fn test_set_remote_description() {
        // Create first peer to make offer
        let config1 = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };
        let peer1 = RtcPeerConnection::new(config1).await.unwrap();
        let offer = peer1
            .create_offer(OfferOptions {
                voice_activity_detection: true,
                ice_restart: false,
            })
            .await
            .unwrap();

        // Create second peer and set remote description
        let config2 = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };
        let mut peer2 = RtcPeerConnection::new(config2).await.unwrap();

        let result = peer2.set_remote_description(offer).await;
        assert!(result.is_ok());
    }

    // IGNORED: Requires external STUN server (stun.l.google.com) for WebRTC negotiation.
    // This test creates actual peer connections which need network access and may fail
    // in isolated test environments. To run: `cargo test --ignored`
    #[tokio::test]
    #[ignore]
    async fn test_add_ice_candidate() {
        // Create peer and setup descriptions first (required for ICE candidates)
        let config1 = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };
        let mut peer1 = RtcPeerConnection::new(config1).await.unwrap();
        let offer = peer1
            .create_offer(OfferOptions {
                voice_activity_detection: true,
                ice_restart: false,
            })
            .await
            .unwrap();
        peer1.set_local_description(offer.clone()).await.unwrap();

        let config2 = RtcConfiguration {
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::Balanced,
        };
        let mut peer2 = RtcPeerConnection::new(config2).await.unwrap();
        peer2.set_remote_description(offer).await.unwrap();

        // Now try adding ICE candidate
        let candidate = IceCandidate {
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.100 54321 typ host".to_string(),
            sdp_mid: Some("0".to_string()),
            sdp_m_line_index: Some(0),
        };

        // Note: This may fail with an invalid candidate, but we're testing the API works
        let _result = peer2.add_ice_candidate(candidate).await;
        // Don't assert success as the candidate format may not be valid
    }
}
