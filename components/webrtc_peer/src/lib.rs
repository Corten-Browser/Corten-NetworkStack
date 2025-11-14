// ! webrtc_peer component
//!
//! WebRTC peer connections, ICE gathering, STUN/TURN, SDP negotiation
//!
//! This component provides WebRTC peer connection functionality including:
//! - RtcPeerConnection for establishing peer-to-peer connections
//! - ICE candidate gathering and processing
//! - SDP offer/answer exchange
//! - STUN/TURN server support
//!
//! # Examples
//!
//! ```no_run
//! use webrtc_peer::{RtcPeerConnection, RtcConfiguration, IceServer, IceTransportPolicy, BundlePolicy};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RtcConfiguration {
//!     ice_servers: vec![IceServer {
//!         urls: vec!["stun:stun.l.google.com:19302".to_string()],
//!         username: None,
//!         credential: None,
//!     }],
//!     ice_transport_policy: IceTransportPolicy::All,
//!     bundle_policy: BundlePolicy::Balanced,
//! };
//!
//! let peer = RtcPeerConnection::new(config).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use interceptor::registry::Registry;
use network_errors::{NetworkError, NetworkResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection as WebRTCPeerConnection;

/// ICE transport policy
///
/// Controls which ICE candidates are gathered and used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceTransportPolicy {
    /// Use all available ICE candidates (host, srflx, relay)
    All,
    /// Only use relay (TURN) candidates
    Relay,
}

/// Bundle policy
///
/// Controls how media streams are bundled together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BundlePolicy {
    /// Bundle all media streams together
    MaxBundle,
    /// Balance between bundling and separate streams
    Balanced,
    /// Use maximum compatibility (separate streams)
    MaxCompat,
}

/// SDP type enumeration
///
/// Represents the type of Session Description Protocol message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdpType {
    /// SDP offer
    Offer,
    /// SDP answer
    Answer,
    /// Provisional answer
    Pranswer,
    /// Rollback
    Rollback,
}

/// ICE server configuration
///
/// Specifies a STUN or TURN server for ICE candidate gathering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    /// List of URLs for the ICE server
    pub urls: Vec<String>,
    /// Username for TURN authentication
    pub username: Option<String>,
    /// Credential for TURN authentication
    pub credential: Option<String>,
}

/// RTC configuration
///
/// Configuration options for creating an RtcPeerConnection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtcConfiguration {
    /// List of ICE servers (STUN/TURN)
    pub ice_servers: Vec<IceServer>,
    /// Policy for ICE transport
    pub ice_transport_policy: IceTransportPolicy,
    /// Policy for bundling media streams
    pub bundle_policy: BundlePolicy,
}

/// Session description
///
/// Represents an SDP (Session Description Protocol) message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    /// Type of session description (offer/answer/etc.)
    pub sdp_type: SdpType,
    /// SDP string content
    pub sdp: String,
}

/// ICE candidate
///
/// Represents an ICE (Interactive Connectivity Establishment) candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    /// Candidate string in ICE format
    pub candidate: String,
    /// Media stream identification tag
    pub sdp_mid: Option<String>,
    /// Media line index
    pub sdp_m_line_index: Option<u16>,
}

/// Offer options
///
/// Options for creating an SDP offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferOptions {
    /// Enable voice activity detection
    pub voice_activity_detection: bool,
    /// Restart ICE gathering
    pub ice_restart: bool,
}

/// Answer options
///
/// Options for creating an SDP answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerOptions {}

/// WebRTC peer connection
///
/// Manages a WebRTC peer-to-peer connection with another endpoint.
///
/// # Examples
///
/// ```no_run
/// use webrtc_peer::{RtcPeerConnection, RtcConfiguration, IceServer, IceTransportPolicy, BundlePolicy, OfferOptions};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = RtcConfiguration {
///     ice_servers: vec![IceServer {
///         urls: vec!["stun:stun.l.google.com:19302".to_string()],
///         username: None,
///         credential: None,
///     }],
///     ice_transport_policy: IceTransportPolicy::All,
///     bundle_policy: BundlePolicy::Balanced,
/// };
///
/// let peer = RtcPeerConnection::new(config).await?;
///
/// let offer = peer.create_offer(OfferOptions {
///     voice_activity_detection: true,
///     ice_restart: false,
/// }).await?;
/// # Ok(())
/// # }
/// ```
pub struct RtcPeerConnection {
    /// Unique connection identifier
    pub connection_id: Uuid,
    /// Configuration
    configuration: RtcConfiguration,
    /// Underlying WebRTC peer connection
    peer_connection: Arc<WebRTCPeerConnection>,
    /// Local description
    local_description: Arc<Mutex<Option<SessionDescription>>>,
    /// Remote description
    remote_description: Arc<Mutex<Option<SessionDescription>>>,
}

impl RtcPeerConnection {
    /// Create a new RTC peer connection
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the peer connection
    ///
    /// # Returns
    ///
    /// Returns a Result containing the new RtcPeerConnection or a NetworkError
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webrtc_peer::{RtcPeerConnection, RtcConfiguration, IceServer, IceTransportPolicy, BundlePolicy};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = RtcConfiguration {
    ///     ice_servers: vec![IceServer {
    ///         urls: vec!["stun:stun.l.google.com:19302".to_string()],
    ///         username: None,
    ///         credential: None,
    ///     }],
    ///     ice_transport_policy: IceTransportPolicy::All,
    ///     bundle_policy: BundlePolicy::Balanced,
    /// };
    ///
    /// let peer = RtcPeerConnection::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: RtcConfiguration) -> NetworkResult<Self> {
        // Create media engine
        let mut media_engine = MediaEngine::default();

        // Create interceptor registry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine).map_err(|e| {
            NetworkError::WebRtcError(format!("Failed to register interceptors: {}", e))
        })?;

        // Create API
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        // Convert our config to WebRTC config
        let rtc_config = Self::convert_config(&config);

        // Create peer connection
        let peer_connection = Arc::new(api.new_peer_connection(rtc_config).await.map_err(|e| {
            NetworkError::WebRtcError(format!("Failed to create peer connection: {}", e))
        })?);

        Ok(Self {
            connection_id: Uuid::new_v4(),
            configuration: config,
            peer_connection,
            local_description: Arc::new(Mutex::new(None)),
            remote_description: Arc::new(Mutex::new(None)),
        })
    }

    /// Create an SDP offer
    ///
    /// # Arguments
    ///
    /// * `options` - Options for creating the offer
    ///
    /// # Returns
    ///
    /// Returns a Result containing the SessionDescription offer or a NetworkError
    pub async fn create_offer(&self, _options: OfferOptions) -> NetworkResult<SessionDescription> {
        let offer = self
            .peer_connection
            .create_offer(None)
            .await
            .map_err(|e| NetworkError::WebRtcError(format!("Failed to create offer: {}", e)))?;

        Ok(SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: offer.sdp,
        })
    }

    /// Create an SDP answer
    ///
    /// # Arguments
    ///
    /// * `options` - Options for creating the answer
    ///
    /// # Returns
    ///
    /// Returns a Result containing the SessionDescription answer or a NetworkError
    pub async fn create_answer(
        &self,
        _options: AnswerOptions,
    ) -> NetworkResult<SessionDescription> {
        let answer = self
            .peer_connection
            .create_answer(None)
            .await
            .map_err(|e| NetworkError::WebRtcError(format!("Failed to create answer: {}", e)))?;

        Ok(SessionDescription {
            sdp_type: SdpType::Answer,
            sdp: answer.sdp,
        })
    }

    /// Set local description
    ///
    /// # Arguments
    ///
    /// * `description` - The local session description to set
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success or a NetworkError
    pub async fn set_local_description(
        &mut self,
        description: SessionDescription,
    ) -> NetworkResult<()> {
        let rtc_desc = match description.sdp_type {
            SdpType::Offer => RTCSessionDescription::offer(description.sdp.clone())
                .map_err(|e| NetworkError::WebRtcError(format!("Failed to create offer: {}", e)))?,
            SdpType::Answer => {
                RTCSessionDescription::answer(description.sdp.clone()).map_err(|e| {
                    NetworkError::WebRtcError(format!("Failed to create answer: {}", e))
                })?
            }
            SdpType::Pranswer => {
                RTCSessionDescription::pranswer(description.sdp.clone()).map_err(|e| {
                    NetworkError::WebRtcError(format!("Failed to create pranswer: {}", e))
                })?
            }
            SdpType::Rollback => {
                return Err(NetworkError::WebRtcError(
                    "Rollback not supported in this context".to_string(),
                ));
            }
        };

        self.peer_connection
            .set_local_description(rtc_desc)
            .await
            .map_err(|e| {
                NetworkError::WebRtcError(format!("Failed to set local description: {}", e))
            })?;

        *self.local_description.lock().await = Some(description);

        Ok(())
    }

    /// Set remote description
    ///
    /// # Arguments
    ///
    /// * `description` - The remote session description to set
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success or a NetworkError
    pub async fn set_remote_description(
        &mut self,
        description: SessionDescription,
    ) -> NetworkResult<()> {
        let rtc_desc = match description.sdp_type {
            SdpType::Offer => RTCSessionDescription::offer(description.sdp.clone())
                .map_err(|e| NetworkError::WebRtcError(format!("Failed to create offer: {}", e)))?,
            SdpType::Answer => {
                RTCSessionDescription::answer(description.sdp.clone()).map_err(|e| {
                    NetworkError::WebRtcError(format!("Failed to create answer: {}", e))
                })?
            }
            SdpType::Pranswer => {
                RTCSessionDescription::pranswer(description.sdp.clone()).map_err(|e| {
                    NetworkError::WebRtcError(format!("Failed to create pranswer: {}", e))
                })?
            }
            SdpType::Rollback => {
                return Err(NetworkError::WebRtcError(
                    "Rollback not supported in this context".to_string(),
                ));
            }
        };

        self.peer_connection
            .set_remote_description(rtc_desc)
            .await
            .map_err(|e| {
                NetworkError::WebRtcError(format!("Failed to set remote description: {}", e))
            })?;

        *self.remote_description.lock().await = Some(description);

        Ok(())
    }

    /// Add an ICE candidate
    ///
    /// # Arguments
    ///
    /// * `candidate` - The ICE candidate to add
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success or a NetworkError
    pub async fn add_ice_candidate(&mut self, candidate: IceCandidate) -> NetworkResult<()> {
        let ice_candidate = webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
            candidate: candidate.candidate,
            sdp_mid: candidate.sdp_mid,
            sdp_mline_index: candidate.sdp_m_line_index,
            username_fragment: None,
        };

        self.peer_connection
            .add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| {
                NetworkError::WebRtcError(format!("Failed to add ICE candidate: {}", e))
            })?;

        Ok(())
    }

    /// Get connection state
    ///
    /// # Returns
    ///
    /// Returns the current connection state
    pub fn connection_state(&self) -> RTCPeerConnectionState {
        self.peer_connection.connection_state()
    }

    /// Close the peer connection
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success or a NetworkError
    pub async fn close(&self) -> NetworkResult<()> {
        self.peer_connection.close().await.map_err(|e| {
            NetworkError::WebRtcError(format!("Failed to close peer connection: {}", e))
        })?;

        Ok(())
    }

    // Helper function to convert our config to WebRTC config
    fn convert_config(config: &RtcConfiguration) -> RTCConfiguration {
        let ice_servers: Vec<RTCIceServer> = config
            .ice_servers
            .iter()
            .map(|server| RTCIceServer {
                urls: server.urls.clone(),
                username: server.username.clone().unwrap_or_default(),
                credential: server.credential.clone().unwrap_or_default(),
                credential_type: Default::default(),
            })
            .collect();

        RTCConfiguration {
            ice_servers,
            ..Default::default()
        }
    }
}
