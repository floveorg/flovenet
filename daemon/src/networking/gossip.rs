use libp2p::gossipsub::{self, IdentTopic as Topic, MessageAuthenticity, MessageId};

pub const TOPIC_SLOTS_ANNOUNCE: &str = "slots/announce";
pub const TOPIC_NODE_STATUS: &str = "node/status";
pub const TOPIC_REPUTATION: &str = "reputation/score";
pub const TOPIC_TRUST_EDGE: &str = "trust/edge";
pub const TOPIC_SOCIAL_POST: &str = "social/post";
pub const TOPIC_SOCIAL_PROFILE: &str = "social/profile";
pub const TOPIC_SOCIAL_FOLLOW: &str = "social/follow";

pub fn create_gossipsub() -> gossipsub::Behaviour {
    let msg_id_fn = |msg: &gossipsub::Message| {
        let data = &msg.data[..msg.data.len().min(64)];
        let hash = quickhash::xxh3_64(data);
        MessageId::from(hash.to_le_bytes().to_vec())
    };

    let config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(std::time::Duration::from_secs(10))
        .message_id_fn(msg_id_fn)
        .validation_mode(gossipsub::ValidationMode::Anonymous)
        .build()
        .expect("gossipsub config");

    gossipsub::Behaviour::new(MessageAuthenticity::Anonymous, config)
        .expect("gossipsub behaviour")
}

pub fn announce_topic() -> Topic {
    Topic::new(TOPIC_SLOTS_ANNOUNCE)
}

pub fn status_topic() -> Topic {
    Topic::new(TOPIC_NODE_STATUS)
}

pub fn social_post_topic() -> Topic {
    Topic::new(TOPIC_SOCIAL_POST)
}

pub fn social_profile_topic() -> Topic {
    Topic::new(TOPIC_SOCIAL_PROFILE)
}

pub fn social_follow_topic() -> Topic {
    Topic::new(TOPIC_SOCIAL_FOLLOW)
}

pub fn reputation_topic() -> Topic {
    Topic::new(TOPIC_REPUTATION)
}

pub fn trust_edge_topic() -> Topic {
    Topic::new(TOPIC_TRUST_EDGE)
}

/// All topics the node subscribes to
pub fn all_topics() -> Vec<Topic> {
    vec![
        announce_topic(),
        status_topic(),
        reputation_topic(),
        trust_edge_topic(),
        social_post_topic(),
        social_profile_topic(),
        social_follow_topic(),
    ]
}

/// Module for quick hash used by gossipsub message ID
mod quickhash {
    pub fn xxh3_64(data: &[u8]) -> u64 {
        let mut h = 0u64;
        for chunk in data.chunks(8) {
            let mut buf = [0u8; 8];
            for (i, b) in chunk.iter().enumerate() {
                buf[i] = *b;
            }
            h ^= u64::from_le_bytes(buf);
            h = h.wrapping_mul(0x9E3779B185EBCA87);
        }
        h
    }
}
