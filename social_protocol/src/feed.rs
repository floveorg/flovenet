use crate::post::Post;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub post: Post,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub items: Vec<FeedItem>,
    pub offset: u32,
    pub limit: u32,
    pub total: u32,
}
