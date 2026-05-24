pub mod discovery;
pub mod gossip;
pub mod swarm;

pub use swarm::NodeNetwork;

#[allow(dead_code)]
pub const GATEWAY_KEY: &str = "/flovenet/gateway/1.0.0";
#[allow(dead_code)]
pub const GATEWAY_EXPIRE_SECS: u64 = 120;
pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Load an optional swarm key file (PSK) for private sub-network support.
/// The key file should contain 32 bytes of raw key material.
/// Returns None if the path is empty or the file cannot be read.
pub fn load_swarm_key(path: Option<&str>) -> Option<[u8; 32]> {
    let path = path?;
    let data = std::fs::read(path).ok()?;
    let mut key = [0u8; 32];
    if data.len() >= 32 {
        key.copy_from_slice(&data[..32]);
        Some(key)
    } else {
        None
    }
}
