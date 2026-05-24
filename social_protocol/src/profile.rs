use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub peer_id: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_cid: Option<String>,
    pub follower_count: u32,
    pub following_count: u32,
    pub post_count: u32,
    pub reputation: Option<f64>,
    pub public_key: Vec<u8>,
}
