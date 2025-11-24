//! WebRTC statistics types and collection
//!
//! This module provides types for collecting and representing WebRTC connection statistics
//! including RTP stream stats, ICE candidate pair stats, and connection quality metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of RTC statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RtcStatsType {
    /// Inbound RTP stream statistics
    InboundRtp,
    /// Outbound RTP stream statistics
    OutboundRtp,
    /// Remote inbound RTP statistics (from RTCP receiver reports)
    RemoteInboundRtp,
    /// ICE candidate pair statistics
    CandidatePair,
    /// Local ICE candidate statistics
    LocalCandidate,
    /// Remote ICE candidate statistics
    RemoteCandidate,
    /// Transport statistics
    Transport,
    /// Codec statistics
    Codec,
    /// Unknown or unsupported statistics type
    Unknown(String),
}

/// Base statistics common to all stat types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtcStatsBase {
    /// Unique identifier for this stats object
    pub id: String,
    /// Timestamp when these stats were collected (milliseconds since epoch)
    pub timestamp: f64,
    /// Type of statistics
    pub stats_type: RtcStatsType,
}

/// Inbound RTP stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundRtpStats {
    /// Base statistics
    #[serde(flatten)]
    pub base: RtcStatsBase,
    /// SSRC identifier for this stream
    pub ssrc: u32,
    /// Media type (audio or video)
    pub kind: String,
    /// Number of packets received
    pub packets_received: u64,
    /// Number of bytes received
    pub bytes_received: u64,
    /// Number of packets lost
    pub packets_lost: i64,
    /// Jitter in seconds
    pub jitter: f64,
    /// Fraction of packets lost (0.0 to 1.0)
    pub fraction_lost: Option<f64>,
}

/// Outbound RTP stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundRtpStats {
    /// Base statistics
    #[serde(flatten)]
    pub base: RtcStatsBase,
    /// SSRC identifier for this stream
    pub ssrc: u32,
    /// Media type (audio or video)
    pub kind: String,
    /// Number of packets sent
    pub packets_sent: u64,
    /// Number of bytes sent
    pub bytes_sent: u64,
    /// Target bitrate in bits per second
    pub target_bitrate: Option<f64>,
}

/// Remote inbound RTP statistics (from RTCP receiver reports)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInboundRtpStats {
    /// Base statistics
    #[serde(flatten)]
    pub base: RtcStatsBase,
    /// SSRC identifier for this stream
    pub ssrc: u32,
    /// Media type (audio or video)
    pub kind: String,
    /// Number of packets lost as reported by remote
    pub packets_lost: i64,
    /// Round trip time in seconds
    pub round_trip_time: Option<f64>,
    /// Jitter as reported by remote (seconds)
    pub jitter: Option<f64>,
    /// Fraction of packets lost (0.0 to 1.0)
    pub fraction_lost: Option<f64>,
}

/// State of an ICE candidate pair
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidatePairState {
    /// Pair is frozen (not yet checked)
    Frozen,
    /// Pair is waiting for check
    Waiting,
    /// Pair is being checked
    InProgress,
    /// Check failed
    Failed,
    /// Check succeeded
    Succeeded,
}

/// ICE candidate pair statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePairStats {
    /// Base statistics
    #[serde(flatten)]
    pub base: RtcStatsBase,
    /// Local candidate ID
    pub local_candidate_id: String,
    /// Remote candidate ID
    pub remote_candidate_id: String,
    /// Current state of the pair
    pub state: CandidatePairState,
    /// Whether this pair is nominated
    pub nominated: bool,
    /// Number of bytes sent on this pair
    pub bytes_sent: u64,
    /// Number of bytes received on this pair
    pub bytes_received: u64,
    /// Total round trip time in seconds
    pub total_round_trip_time: f64,
    /// Current round trip time in seconds
    pub current_round_trip_time: Option<f64>,
    /// Available outgoing bitrate estimate
    pub available_outgoing_bitrate: Option<f64>,
    /// Available incoming bitrate estimate
    pub available_incoming_bitrate: Option<f64>,
}

/// Type of ICE candidate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceCandidateType {
    /// Host candidate (local address)
    Host,
    /// Server reflexive candidate (from STUN)
    Srflx,
    /// Peer reflexive candidate
    Prflx,
    /// Relay candidate (from TURN)
    Relay,
    /// Unknown candidate type
    Unknown(String),
}

/// ICE candidate statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidateStats {
    /// Base statistics
    #[serde(flatten)]
    pub base: RtcStatsBase,
    /// Transport ID
    pub transport_id: String,
    /// IP address
    pub address: Option<String>,
    /// Port number
    pub port: Option<u16>,
    /// Protocol (udp or tcp)
    pub protocol: String,
    /// Candidate type
    pub candidate_type: IceCandidateType,
    /// Priority of this candidate
    pub priority: Option<u32>,
    /// URL of the server used to obtain this candidate
    pub url: Option<String>,
}

/// Transport statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStats {
    /// Base statistics
    #[serde(flatten)]
    pub base: RtcStatsBase,
    /// Number of bytes sent
    pub bytes_sent: u64,
    /// Number of bytes received
    pub bytes_received: u64,
    /// DTLS state
    pub dtls_state: Option<String>,
    /// Selected candidate pair ID
    pub selected_candidate_pair_id: Option<String>,
    /// Local certificate ID
    pub local_certificate_id: Option<String>,
    /// Remote certificate ID
    pub remote_certificate_id: Option<String>,
}

/// A single RTC statistics entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RtcStats {
    /// Inbound RTP statistics
    InboundRtp(InboundRtpStats),
    /// Outbound RTP statistics
    OutboundRtp(OutboundRtpStats),
    /// Remote inbound RTP statistics
    RemoteInboundRtp(RemoteInboundRtpStats),
    /// Candidate pair statistics
    CandidatePair(CandidatePairStats),
    /// Local candidate statistics
    LocalCandidate(IceCandidateStats),
    /// Remote candidate statistics
    RemoteCandidate(IceCandidateStats),
    /// Transport statistics
    Transport(TransportStats),
}

/// Complete statistics report from a peer connection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RtcStatsReport {
    /// All statistics entries indexed by their ID
    pub stats: HashMap<String, RtcStats>,
}

impl RtcStatsReport {
    /// Create a new empty stats report
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Add a stats entry to the report
    pub fn add(&mut self, id: String, stats: RtcStats) {
        self.stats.insert(id, stats);
    }

    /// Get inbound RTP stats
    pub fn inbound_rtp_stats(&self) -> Vec<&InboundRtpStats> {
        self.stats
            .values()
            .filter_map(|s| match s {
                RtcStats::InboundRtp(stats) => Some(stats),
                _ => None,
            })
            .collect()
    }

    /// Get outbound RTP stats
    pub fn outbound_rtp_stats(&self) -> Vec<&OutboundRtpStats> {
        self.stats
            .values()
            .filter_map(|s| match s {
                RtcStats::OutboundRtp(stats) => Some(stats),
                _ => None,
            })
            .collect()
    }

    /// Get remote inbound RTP stats
    pub fn remote_inbound_rtp_stats(&self) -> Vec<&RemoteInboundRtpStats> {
        self.stats
            .values()
            .filter_map(|s| match s {
                RtcStats::RemoteInboundRtp(stats) => Some(stats),
                _ => None,
            })
            .collect()
    }

    /// Get candidate pair stats
    pub fn candidate_pair_stats(&self) -> Vec<&CandidatePairStats> {
        self.stats
            .values()
            .filter_map(|s| match s {
                RtcStats::CandidatePair(stats) => Some(stats),
                _ => None,
            })
            .collect()
    }

    /// Get the nominated (active) candidate pair
    pub fn nominated_candidate_pair(&self) -> Option<&CandidatePairStats> {
        self.candidate_pair_stats()
            .into_iter()
            .find(|pair| pair.nominated)
    }

    /// Get transport stats
    pub fn transport_stats(&self) -> Vec<&TransportStats> {
        self.stats
            .values()
            .filter_map(|s| match s {
                RtcStats::Transport(stats) => Some(stats),
                _ => None,
            })
            .collect()
    }
}

/// Connection quality metrics derived from statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuality {
    /// Current round trip time in milliseconds
    pub rtt_ms: Option<f64>,
    /// Jitter in milliseconds
    pub jitter_ms: Option<f64>,
    /// Packet loss percentage (0.0 to 100.0)
    pub packet_loss_percent: Option<f64>,
    /// Current bitrate in bits per second (sending)
    pub outgoing_bitrate_bps: Option<f64>,
    /// Current bitrate in bits per second (receiving)
    pub incoming_bitrate_bps: Option<f64>,
    /// Quality score (0.0 to 1.0, where 1.0 is excellent)
    pub quality_score: f64,
}

impl ConnectionQuality {
    /// Calculate connection quality from a stats report
    pub fn from_stats_report(report: &RtcStatsReport) -> Self {
        let mut rtt_ms = None;
        let mut jitter_ms = None;
        let mut packet_loss_percent = None;
        let mut outgoing_bitrate_bps = None;
        let mut incoming_bitrate_bps = None;

        // Get RTT and available bitrates from nominated candidate pair
        if let Some(pair) = report.nominated_candidate_pair() {
            rtt_ms = pair.current_round_trip_time.map(|rtt| rtt * 1000.0);
            outgoing_bitrate_bps = pair.available_outgoing_bitrate;
            incoming_bitrate_bps = pair.available_incoming_bitrate;
        }

        // Get jitter and packet loss from remote inbound stats
        for remote_stats in report.remote_inbound_rtp_stats() {
            if jitter_ms.is_none() {
                jitter_ms = remote_stats.jitter.map(|j| j * 1000.0);
            }
            if packet_loss_percent.is_none() {
                packet_loss_percent = remote_stats.fraction_lost.map(|f| f * 100.0);
            }
        }

        // Calculate quality score
        let quality_score = Self::calculate_quality_score(rtt_ms, jitter_ms, packet_loss_percent);

        Self {
            rtt_ms,
            jitter_ms,
            packet_loss_percent,
            outgoing_bitrate_bps,
            incoming_bitrate_bps,
            quality_score,
        }
    }

    /// Calculate a quality score from 0.0 (poor) to 1.0 (excellent)
    fn calculate_quality_score(
        rtt_ms: Option<f64>,
        jitter_ms: Option<f64>,
        packet_loss_percent: Option<f64>,
    ) -> f64 {
        let mut score = 1.0;

        // Penalize high RTT (> 300ms is poor)
        if let Some(rtt) = rtt_ms {
            let rtt_factor = (1.0 - (rtt / 500.0).min(1.0)).max(0.0);
            score *= rtt_factor;
        }

        // Penalize high jitter (> 50ms is poor)
        if let Some(jitter) = jitter_ms {
            let jitter_factor = (1.0 - (jitter / 100.0).min(1.0)).max(0.0);
            score *= jitter_factor;
        }

        // Penalize packet loss (> 5% is poor)
        if let Some(loss) = packet_loss_percent {
            let loss_factor = (1.0 - (loss / 10.0).min(1.0)).max(0.0);
            score *= loss_factor;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtc_stats_report_new() {
        let report = RtcStatsReport::new();
        assert!(report.stats.is_empty());
    }

    #[test]
    fn test_rtc_stats_report_add_and_retrieve() {
        let mut report = RtcStatsReport::new();

        let inbound = InboundRtpStats {
            base: RtcStatsBase {
                id: "inbound-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::InboundRtp,
            },
            ssrc: 12345,
            kind: "video".to_string(),
            packets_received: 1000,
            bytes_received: 100000,
            packets_lost: 5,
            jitter: 0.01,
            fraction_lost: Some(0.005),
        };

        report.add("inbound-1".to_string(), RtcStats::InboundRtp(inbound));

        assert_eq!(report.stats.len(), 1);
        assert_eq!(report.inbound_rtp_stats().len(), 1);
    }

    #[test]
    fn test_inbound_rtp_stats_filtering() {
        let mut report = RtcStatsReport::new();

        let inbound = InboundRtpStats {
            base: RtcStatsBase {
                id: "inbound-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::InboundRtp,
            },
            ssrc: 12345,
            kind: "audio".to_string(),
            packets_received: 500,
            bytes_received: 50000,
            packets_lost: 2,
            jitter: 0.005,
            fraction_lost: Some(0.004),
        };

        let outbound = OutboundRtpStats {
            base: RtcStatsBase {
                id: "outbound-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::OutboundRtp,
            },
            ssrc: 54321,
            kind: "video".to_string(),
            packets_sent: 800,
            bytes_sent: 80000,
            target_bitrate: Some(1000000.0),
        };

        report.add("inbound-1".to_string(), RtcStats::InboundRtp(inbound));
        report.add("outbound-1".to_string(), RtcStats::OutboundRtp(outbound));

        assert_eq!(report.inbound_rtp_stats().len(), 1);
        assert_eq!(report.outbound_rtp_stats().len(), 1);
    }

    #[test]
    fn test_nominated_candidate_pair() {
        let mut report = RtcStatsReport::new();

        let pair1 = CandidatePairStats {
            base: RtcStatsBase {
                id: "pair-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::CandidatePair,
            },
            local_candidate_id: "local-1".to_string(),
            remote_candidate_id: "remote-1".to_string(),
            state: CandidatePairState::Succeeded,
            nominated: false,
            bytes_sent: 1000,
            bytes_received: 2000,
            total_round_trip_time: 0.1,
            current_round_trip_time: Some(0.05),
            available_outgoing_bitrate: Some(1000000.0),
            available_incoming_bitrate: Some(1500000.0),
        };

        let pair2 = CandidatePairStats {
            base: RtcStatsBase {
                id: "pair-2".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::CandidatePair,
            },
            local_candidate_id: "local-2".to_string(),
            remote_candidate_id: "remote-2".to_string(),
            state: CandidatePairState::Succeeded,
            nominated: true,
            bytes_sent: 5000,
            bytes_received: 6000,
            total_round_trip_time: 0.08,
            current_round_trip_time: Some(0.04),
            available_outgoing_bitrate: Some(2000000.0),
            available_incoming_bitrate: Some(2500000.0),
        };

        report.add("pair-1".to_string(), RtcStats::CandidatePair(pair1));
        report.add("pair-2".to_string(), RtcStats::CandidatePair(pair2));

        let nominated = report.nominated_candidate_pair();
        assert!(nominated.is_some());
        assert_eq!(nominated.unwrap().base.id, "pair-2");
    }

    #[test]
    fn test_connection_quality_excellent() {
        let mut report = RtcStatsReport::new();

        let pair = CandidatePairStats {
            base: RtcStatsBase {
                id: "pair-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::CandidatePair,
            },
            local_candidate_id: "local-1".to_string(),
            remote_candidate_id: "remote-1".to_string(),
            state: CandidatePairState::Succeeded,
            nominated: true,
            bytes_sent: 1000,
            bytes_received: 2000,
            total_round_trip_time: 0.05,
            current_round_trip_time: Some(0.025), // 25ms RTT
            available_outgoing_bitrate: Some(5000000.0),
            available_incoming_bitrate: Some(5000000.0),
        };

        let remote = RemoteInboundRtpStats {
            base: RtcStatsBase {
                id: "remote-inbound-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::RemoteInboundRtp,
            },
            ssrc: 12345,
            kind: "video".to_string(),
            packets_lost: 1,
            round_trip_time: Some(0.025),
            jitter: Some(0.005), // 5ms jitter
            fraction_lost: Some(0.001), // 0.1% loss
        };

        report.add("pair-1".to_string(), RtcStats::CandidatePair(pair));
        report.add(
            "remote-inbound-1".to_string(),
            RtcStats::RemoteInboundRtp(remote),
        );

        let quality = ConnectionQuality::from_stats_report(&report);

        assert!(quality.rtt_ms.is_some());
        assert!((quality.rtt_ms.unwrap() - 25.0).abs() < 0.01);
        assert!(quality.jitter_ms.is_some());
        assert!((quality.jitter_ms.unwrap() - 5.0).abs() < 0.01);
        assert!(quality.packet_loss_percent.is_some());
        assert!((quality.packet_loss_percent.unwrap() - 0.1).abs() < 0.01);
        assert!(quality.quality_score > 0.8); // Should be excellent
    }

    #[test]
    fn test_connection_quality_poor() {
        let mut report = RtcStatsReport::new();

        let pair = CandidatePairStats {
            base: RtcStatsBase {
                id: "pair-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::CandidatePair,
            },
            local_candidate_id: "local-1".to_string(),
            remote_candidate_id: "remote-1".to_string(),
            state: CandidatePairState::Succeeded,
            nominated: true,
            bytes_sent: 1000,
            bytes_received: 2000,
            total_round_trip_time: 0.5,
            current_round_trip_time: Some(0.4), // 400ms RTT (very high)
            available_outgoing_bitrate: Some(100000.0),
            available_incoming_bitrate: Some(100000.0),
        };

        let remote = RemoteInboundRtpStats {
            base: RtcStatsBase {
                id: "remote-inbound-1".to_string(),
                timestamp: 1234567890.0,
                stats_type: RtcStatsType::RemoteInboundRtp,
            },
            ssrc: 12345,
            kind: "video".to_string(),
            packets_lost: 100,
            round_trip_time: Some(0.4),
            jitter: Some(0.08), // 80ms jitter (high)
            fraction_lost: Some(0.08), // 8% loss (high)
        };

        report.add("pair-1".to_string(), RtcStats::CandidatePair(pair));
        report.add(
            "remote-inbound-1".to_string(),
            RtcStats::RemoteInboundRtp(remote),
        );

        let quality = ConnectionQuality::from_stats_report(&report);

        assert!(quality.quality_score < 0.3); // Should be poor
    }

    #[test]
    fn test_connection_quality_empty_report() {
        let report = RtcStatsReport::new();
        let quality = ConnectionQuality::from_stats_report(&report);

        assert!(quality.rtt_ms.is_none());
        assert!(quality.jitter_ms.is_none());
        assert!(quality.packet_loss_percent.is_none());
        assert!((quality.quality_score - 1.0).abs() < 0.01); // No data = max score
    }

    #[test]
    fn test_ice_candidate_type_variants() {
        assert_eq!(IceCandidateType::Host, IceCandidateType::Host);
        assert_ne!(IceCandidateType::Host, IceCandidateType::Srflx);
        assert_eq!(
            IceCandidateType::Unknown("test".to_string()),
            IceCandidateType::Unknown("test".to_string())
        );
    }

    #[test]
    fn test_candidate_pair_state_variants() {
        assert_eq!(CandidatePairState::Frozen, CandidatePairState::Frozen);
        assert_ne!(CandidatePairState::Waiting, CandidatePairState::InProgress);
        assert_eq!(CandidatePairState::Succeeded, CandidatePairState::Succeeded);
    }

    #[test]
    fn test_rtc_stats_type_variants() {
        assert_eq!(RtcStatsType::InboundRtp, RtcStatsType::InboundRtp);
        assert_ne!(RtcStatsType::OutboundRtp, RtcStatsType::InboundRtp);
        assert_eq!(
            RtcStatsType::Unknown("custom".to_string()),
            RtcStatsType::Unknown("custom".to_string())
        );
    }
}
