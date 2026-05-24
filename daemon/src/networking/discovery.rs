use serde::{Deserialize, Serialize};

/// Payload published to DHT for gateway discovery
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayAnnounce {
    pub peer_id: String,
    pub api_url: String,
    pub region: String,
    pub roles: Vec<String>,
    pub reputation_score: f64,
    pub ttl_secs: u64,
    pub timestamp_secs: u64,
}

/// Result of a DHT gateway lookup
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub peer_id: String,
    pub api_url: String,
    pub region: String,
    pub roles: Vec<String>,
    pub reputation_score: f64,
    pub latency_ms: Option<u64>,
}
