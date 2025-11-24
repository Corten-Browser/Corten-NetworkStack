// ! webrtc_peer component
//!
//! WebRTC peer connections, ICE gathering, STUN/TURN, SDP negotiation
//!
//! This component provides WebRTC peer connection functionality including:
//! - RtcPeerConnection for establishing peer-to-peer connections
//! - ICE candidate gathering and processing
//! - SDP offer/answer exchange
//! - STUN/TURN server support
//! - Media track support (audio/video) with SRTP/RTP/RTCP
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

pub mod stats;

pub use stats::{
    CandidatePairState, CandidatePairStats, ConnectionQuality, IceCandidateStats,
    IceCandidateType, InboundRtpStats, OutboundRtpStats, RemoteInboundRtpStats, RtcStats,
    RtcStatsBase, RtcStatsReport, RtcStatsType, TransportStats,
};

use interceptor::registry::Registry;
use network_errors::{NetworkError, NetworkResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTPCodecType};
use webrtc::rtp_transceiver::rtp_sender::RTCRtpSender;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;
use webrtc::track::track_remote::TrackRemote;

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

/// Media track kind
///
/// Represents the type of media in a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaTrackKind {
    /// Audio track
    Audio,
    /// Video track
    Video,
}

/// Codec capability for media tracks
///
/// Defines the codec parameters for audio or video encoding/decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecCapability {
    /// MIME type of the codec (e.g., "audio/opus", "video/VP8")
    pub mime_type: String,
    /// Clock rate in Hz (e.g., 48000 for Opus, 90000 for video)
    pub clock_rate: u32,
    /// Number of audio channels (only for audio codecs)
    pub channels: Option<u16>,
    /// SDP format-specific parameters (fmtp line)
    pub sdp_fmtp_line: Option<String>,
}

impl CodecCapability {
    /// Create a new audio codec capability for Opus
    pub fn opus() -> Self {
        Self {
            mime_type: "audio/opus".to_string(),
            clock_rate: 48000,
            channels: Some(2),
            sdp_fmtp_line: Some("minptime=10;useinbandfec=1".to_string()),
        }
    }

    /// Create a new video codec capability for VP8
    pub fn vp8() -> Self {
        Self {
            mime_type: "video/VP8".to_string(),
            clock_rate: 90000,
            channels: None,
            sdp_fmtp_line: None,
        }
    }

    /// Create a new video codec capability for VP9
    pub fn vp9() -> Self {
        Self {
            mime_type: "video/VP9".to_string(),
            clock_rate: 90000,
            channels: None,
            sdp_fmtp_line: None,
        }
    }

    /// Create a new video codec capability for H.264
    pub fn h264() -> Self {
        Self {
            mime_type: "video/H264".to_string(),
            clock_rate: 90000,
            channels: None,
            sdp_fmtp_line: Some(
                "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f".to_string(),
            ),
        }
    }

    /// Convert to RTCRtpCodecCapability for internal use
    fn to_rtc_codec_capability(&self) -> RTCRtpCodecCapability {
        RTCRtpCodecCapability {
            mime_type: self.mime_type.clone(),
            clock_rate: self.clock_rate,
            channels: self.channels.unwrap_or(0),
            sdp_fmtp_line: self.sdp_fmtp_line.clone().unwrap_or_default(),
            rtcp_feedback: vec![],
        }
    }
}

/// Media track for adding local media to a peer connection
///
/// Represents a local media track that can be added to a peer connection
/// for sending audio or video data to the remote peer.
#[derive(Debug, Clone)]
pub struct MediaTrack {
    /// Unique identifier for the track
    pub id: String,
    /// Stream identifier this track belongs to
    pub stream_id: String,
    /// Kind of media (audio or video)
    pub kind: MediaTrackKind,
    /// Codec capability for encoding
    pub codec: CodecCapability,
}

impl MediaTrack {
    /// Create a new media track
    pub fn new(id: String, stream_id: String, kind: MediaTrackKind, codec: CodecCapability) -> Self {
        Self { id, stream_id, kind, codec }
    }

    /// Create a new audio track with Opus codec
    pub fn audio(id: impl Into<String>, stream_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            stream_id: stream_id.into(),
            kind: MediaTrackKind::Audio,
            codec: CodecCapability::opus(),
        }
    }

    /// Create a new video track with VP8 codec
    pub fn video_vp8(id: impl Into<String>, stream_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            stream_id: stream_id.into(),
            kind: MediaTrackKind::Video,
            codec: CodecCapability::vp8(),
        }
    }

    /// Create a new video track with H.264 codec
    pub fn video_h264(id: impl Into<String>, stream_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            stream_id: stream_id.into(),
            kind: MediaTrackKind::Video,
            codec: CodecCapability::h264(),
        }
    }
}

/// Remote media stream track
///
/// Represents a media track received from a remote peer.
#[derive(Clone)]
pub struct MediaStreamTrack {
    /// Unique identifier for the track
    pub id: String,
    /// Kind of media (audio or video)
    pub kind: MediaTrackKind,
    /// Stream ID this track belongs to
    pub stream_id: String,
    /// SSRC (Synchronization Source) identifier
    pub ssrc: u32,
    /// Payload type from SDP
    pub payload_type: u8,
    /// Codec MIME type
    pub codec_mime_type: String,
    /// Internal track reference for reading data
    track: Arc<TrackRemote>,
}

impl std::fmt::Debug for MediaStreamTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaStreamTrack")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("stream_id", &self.stream_id)
            .field("ssrc", &self.ssrc)
            .field("payload_type", &self.payload_type)
            .field("codec_mime_type", &self.codec_mime_type)
            .finish()
    }
}

impl MediaStreamTrack {
    /// Create a new MediaStreamTrack from a remote track
    fn from_track_remote(track: Arc<TrackRemote>) -> Self {
        let kind = if track.kind() == RTPCodecType::Audio {
            MediaTrackKind::Audio
        } else {
            MediaTrackKind::Video
        };
        Self {
            id: track.id().to_string(),
            kind,
            stream_id: track.stream_id().to_string(),
            ssrc: track.ssrc(),
            payload_type: track.payload_type(),
            codec_mime_type: track.codec().capability.mime_type.clone(),
            track,
        }
    }

    /// Get the internal track reference for advanced operations
    pub fn inner_track(&self) -> &Arc<TrackRemote> {
        &self.track
    }
}

/// Track event data
///
/// Contains information about a received remote track.
#[derive(Debug, Clone)]
pub struct TrackEvent {
    /// The received track
    pub track: MediaStreamTrack,
    /// Stream IDs associated with this track
    pub streams: Vec<String>,
}

/// RTP sender handle
///
/// Represents a sender for a local track.
pub struct RtpSender {
    /// The underlying RTP sender
    sender: Arc<RTCRtpSender>,
    /// Track ID
    pub track_id: String,
}

impl RtpSender {
    /// Get the underlying sender for advanced operations
    pub fn inner(&self) -> &Arc<RTCRtpSender> {
        &self.sender
    }
}

impl std::fmt::Debug for RtpSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RtpSender")
            .field("track_id", &self.track_id)
            .finish()
    }
}

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
    /// Local tracks added to this connection
    local_tracks: Arc<Mutex<HashMap<String, Arc<TrackLocalStaticRTP>>>>,
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
            local_tracks: Arc::new(Mutex::new(HashMap::new())),
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

    /// Add a media track to the peer connection
    ///
    /// Adds a local media track to be sent to the remote peer.
    ///
    /// # Arguments
    ///
    /// * `track` - The media track to add
    ///
    /// # Returns
    ///
    /// Returns an RtpSender handle for controlling the track
    pub async fn add_track(&self, track: MediaTrack) -> NetworkResult<RtpSender> {
        let local_track = Arc::new(TrackLocalStaticRTP::new(
            track.codec.to_rtc_codec_capability(),
            track.id.clone(),
            track.stream_id.clone(),
        ));

        let sender = self
            .peer_connection
            .add_track(Arc::clone(&local_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| NetworkError::WebRtcError(format!("Failed to add track: {}", e)))?;

        {
            let mut tracks = self.local_tracks.lock().await;
            tracks.insert(track.id.clone(), local_track);
        }

        Ok(RtpSender {
            sender,
            track_id: track.id,
        })
    }

    /// Register a callback for when a remote track is received
    ///
    /// The callback will be invoked whenever the remote peer adds a track.
    ///
    /// # Arguments
    ///
    /// * `callback` - An async closure that receives TrackEvent when a track arrives
    pub async fn on_track<F>(&self, mut callback: F)
    where
        F: FnMut(TrackEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.peer_connection.on_track(Box::new(
            move |track: Arc<TrackRemote>,
                  _receiver: Arc<webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver>,
                  _transceiver: Arc<webrtc::rtp_transceiver::RTCRtpTransceiver>| {
                let media_track = MediaStreamTrack::from_track_remote(track.clone());
                let streams = vec![media_track.stream_id.clone()];
                let event = TrackEvent {
                    track: media_track,
                    streams,
                };
                let fut = callback(event);
                Box::pin(async move {
                    fut.await;
                })
            },
        ));
    }

    /// Get the list of local track IDs
    pub async fn get_local_track_ids(&self) -> Vec<String> {
        let tracks = self.local_tracks.lock().await;
        tracks.keys().cloned().collect()
    }

    /// Remove a local track by ID
    ///
    /// Returns true if the track was found and removed
    pub async fn remove_track(&self, track_id: &str) -> bool {
        let mut tracks = self.local_tracks.lock().await;
        tracks.remove(track_id).is_some()
    }

    /// Get connection state
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

    /// Get connection statistics
    ///
    /// Collects current statistics from the peer connection including
    /// RTP stream stats, ICE candidate pair stats, and transport stats.
    ///
    /// # Returns
    ///
    /// Returns an RtcStatsReport containing all available statistics
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
    /// let stats = peer.get_stats().await;
    ///
    /// for inbound in stats.inbound_rtp_stats() {
    ///     println!("Received {} packets", inbound.packets_received);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_stats(&self) -> RtcStatsReport {
        let mut report = RtcStatsReport::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        // Get stats from the underlying peer connection
        let webrtc_stats = self.peer_connection.get_stats().await;

        // Convert webrtc stats to our stats types
        for (id, stats) in webrtc_stats.reports {
            match stats {
                webrtc::stats::StatsReportType::InboundRTP(rtp) => {
                    let inbound = InboundRtpStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::InboundRtp,
                        },
                        ssrc: rtp.ssrc,
                        kind: rtp.kind.to_string(),
                        packets_received: rtp.packets_received,
                        bytes_received: rtp.bytes_received as u64,
                        packets_lost: 0, // Not directly available in webrtc-rs
                        jitter: 0.0,     // Not directly available in webrtc-rs
                        fraction_lost: None,
                    };
                    report.add(id, RtcStats::InboundRtp(inbound));
                }
                webrtc::stats::StatsReportType::OutboundRTP(rtp) => {
                    let outbound = OutboundRtpStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::OutboundRtp,
                        },
                        ssrc: rtp.ssrc,
                        kind: rtp.kind.to_string(),
                        packets_sent: rtp.packets_sent,
                        bytes_sent: rtp.bytes_sent as u64,
                        target_bitrate: None,
                    };
                    report.add(id, RtcStats::OutboundRtp(outbound));
                }
                webrtc::stats::StatsReportType::RemoteInboundRTP(rtp) => {
                    let remote = RemoteInboundRtpStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::RemoteInboundRtp,
                        },
                        ssrc: rtp.ssrc,
                        kind: rtp.kind.to_string(),
                        packets_lost: rtp.packets_lost as i64,
                        round_trip_time: rtp.round_trip_time,
                        jitter: None, // Not directly available in webrtc-rs RemoteInboundRTPStats
                        fraction_lost: Some(rtp.fraction_lost),
                    };
                    report.add(id, RtcStats::RemoteInboundRtp(remote));
                }
                webrtc::stats::StatsReportType::CandidatePair(pair) => {
                    let state = if pair.nominated {
                        CandidatePairState::Succeeded
                    } else {
                        CandidatePairState::Waiting
                    };
                    let candidate_pair = CandidatePairStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::CandidatePair,
                        },
                        local_candidate_id: pair.local_candidate_id.clone(),
                        remote_candidate_id: pair.remote_candidate_id.clone(),
                        state,
                        nominated: pair.nominated,
                        bytes_sent: pair.bytes_sent as u64,
                        bytes_received: pair.bytes_received as u64,
                        total_round_trip_time: pair.total_round_trip_time,
                        current_round_trip_time: Some(pair.current_round_trip_time),
                        available_outgoing_bitrate: Some(pair.available_outgoing_bitrate),
                        available_incoming_bitrate: None,
                    };
                    report.add(id, RtcStats::CandidatePair(candidate_pair));
                }
                webrtc::stats::StatsReportType::LocalCandidate(cand) => {
                    let candidate_type_str = format!("{:?}", cand.candidate_type);
                    let candidate_type = match candidate_type_str.as_str() {
                        "Host" => IceCandidateType::Host,
                        "ServerReflexive" => IceCandidateType::Srflx,
                        "PeerReflexive" => IceCandidateType::Prflx,
                        "Relay" => IceCandidateType::Relay,
                        other => IceCandidateType::Unknown(other.to_string()),
                    };
                    let local_cand = IceCandidateStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::LocalCandidate,
                        },
                        transport_id: String::new(),
                        address: Some(cand.ip.clone()),
                        port: Some(cand.port as u16),
                        protocol: String::new(),
                        candidate_type,
                        priority: Some(cand.priority),
                        url: None,
                    };
                    report.add(id, RtcStats::LocalCandidate(local_cand));
                }
                webrtc::stats::StatsReportType::RemoteCandidate(cand) => {
                    let candidate_type_str = format!("{:?}", cand.candidate_type);
                    let candidate_type = match candidate_type_str.as_str() {
                        "Host" => IceCandidateType::Host,
                        "ServerReflexive" => IceCandidateType::Srflx,
                        "PeerReflexive" => IceCandidateType::Prflx,
                        "Relay" => IceCandidateType::Relay,
                        other => IceCandidateType::Unknown(other.to_string()),
                    };
                    let remote_cand = IceCandidateStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::RemoteCandidate,
                        },
                        transport_id: String::new(),
                        address: Some(cand.ip.clone()),
                        port: Some(cand.port as u16),
                        protocol: String::new(),
                        candidate_type,
                        priority: Some(cand.priority),
                        url: None,
                    };
                    report.add(id, RtcStats::RemoteCandidate(remote_cand));
                }
                webrtc::stats::StatsReportType::Transport(transport) => {
                    let transport_stats = TransportStats {
                        base: RtcStatsBase {
                            id: id.clone(),
                            timestamp,
                            stats_type: RtcStatsType::Transport,
                        },
                        bytes_sent: transport.bytes_sent as u64,
                        bytes_received: transport.bytes_received as u64,
                        dtls_state: None,
                        selected_candidate_pair_id: None,
                        local_certificate_id: None,
                        remote_certificate_id: None,
                    };
                    report.add(id, RtcStats::Transport(transport_stats));
                }
                _ => {
                    // Skip unknown or unhandled stats types
                }
            }
        }

        report
    }

    /// Get connection quality metrics
    ///
    /// Convenience method that collects statistics and calculates
    /// connection quality metrics including RTT, jitter, packet loss,
    /// and an overall quality score.
    ///
    /// # Returns
    ///
    /// Returns a ConnectionQuality struct with quality metrics
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
    /// let quality = peer.get_connection_quality().await;
    ///
    /// if quality.quality_score > 0.8 {
    ///     println!("Connection quality is excellent!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_connection_quality(&self) -> ConnectionQuality {
        let report = self.get_stats().await;
        ConnectionQuality::from_stats_report(&report)
    }
}
