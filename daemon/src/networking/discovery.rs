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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_announce_serde() {
        let ga = GatewayAnnounce {
            peer_id: "12D3KooWTest".into(),
            api_url: "http://localhost:8080".into(),
            region: "auto".into(),
            roles: vec!["compute".into(), "storage".into()],
            reputation_score: 50.0,
            ttl_secs: 120,
            timestamp_secs: 1000000,
        };
        let json = serde_json::to_string(&ga).unwrap();
        let decoded: GatewayAnnounce = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.peer_id, "12D3KooWTest");
        assert_eq!(decoded.roles.len(), 2);
        assert!((decoded.reputation_score - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_gateway_info_serde() {
        let gi = GatewayInfo {
            peer_id: "12D3KooWOther".into(),
            api_url: "http://other:9090".into(),
            region: "eu".into(),
            roles: vec!["ai".into()],
            reputation_score: 75.5,
            latency_ms: Some(42),
        };
        let json = serde_json::to_string(&gi).unwrap();
        let decoded: GatewayInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.peer_id, "12D3KooWOther");
        assert_eq!(decoded.latency_ms, Some(42));
        assert!(decoded.roles.contains(&"ai".to_string()));
    }

    #[test]
    fn test_gateway_info_no_latency() {
        let gi = GatewayInfo {
            peer_id: "id".into(),
            api_url: "http://x".into(),
            region: "na".into(),
            roles: vec![],
            reputation_score: 0.0,
            latency_ms: None,
        };
        assert!(gi.latency_ms.is_none());
    }
}
