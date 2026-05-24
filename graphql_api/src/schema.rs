use async_graphql::{Context, Object, SimpleObject, Subscription};
use chrono::{DateTime, Utc};
use futures::Stream;

use crate::AppState;

#[derive(SimpleObject, Clone)]
pub struct AuthPayload {
    pub token: String,
    pub profile: Profile,
}

#[derive(SimpleObject, Clone)]
pub struct Profile {
    pub peer_id: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_cid: Option<String>,
    pub follower_count: u32,
    pub following_count: u32,
    pub post_count: u32,
    pub reputation: Option<f64>,
}

#[derive(SimpleObject, Clone)]
pub struct Post {
    pub cid: String,
    pub author: String,
    pub content: String,
    pub media: Vec<String>,
    pub parent: Option<String>,
    pub reply_count: u32,
    pub like_count: u32,
    pub timestamp: DateTime<Utc>,
    pub signature: Vec<u8>,
}

#[derive(SimpleObject, Clone)]
pub struct GatewayInfo {
    pub peer_id: String,
    pub api_url: String,
    pub region: String,
    pub roles: Vec<String>,
    pub reputation_score: f64,
    pub latency_ms: Option<u64>,
}

#[derive(SimpleObject, Clone)]
pub struct FeedItem {
    pub post: Post,
    pub author: Profile,
    pub score: Option<f64>,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn available_gateways(&self, _ctx: &Context<'_>, _region: Option<String>) -> Vec<GatewayInfo> {
        Vec::new()
    }

    async fn profile(&self, ctx: &Context<'_>, peer_id: String) -> Option<Profile> {
        let state = ctx.data_unchecked::<AppState>();
        state.get_profile(&peer_id).await
    }

    async fn post(&self, ctx: &Context<'_>, cid: String) -> Option<Post> {
        let state = ctx.data_unchecked::<AppState>();
        state.get_post(&cid).await
    }

    async fn feed(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Vec<FeedItem> {
        let state = ctx.data_unchecked::<AppState>();
        state.get_feed(limit.unwrap_or(20) as usize, offset.unwrap_or(0) as usize).await
    }

    async fn search_profiles(&self, ctx: &Context<'_>, query: String) -> Vec<Profile> {
        let state = ctx.data_unchecked::<AppState>();
        state.search_profiles(&query).await
    }

    async fn search_posts(&self, ctx: &Context<'_>, query: String) -> Vec<Post> {
        let state = ctx.data_unchecked::<AppState>();
        state.search_posts(&query).await
    }

    async fn followers(
        &self,
        ctx: &Context<'_>,
        peer_id: String,
        limit: Option<i32>,
    ) -> Vec<Profile> {
        let state = ctx.data_unchecked::<AppState>();
        state.get_followers(&peer_id, limit.unwrap_or(50) as usize).await
    }

    async fn following(
        &self,
        ctx: &Context<'_>,
        peer_id: String,
        limit: Option<i32>,
    ) -> Vec<Profile> {
        let state = ctx.data_unchecked::<AppState>();
        state.get_following(&peer_id, limit.unwrap_or(50) as usize).await
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
        display_name: String,
    ) -> async_graphql::Result<AuthPayload> {
        let state = ctx.data_unchecked::<AppState>();
        state.register_user(&email, &password, &display_name).await
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> async_graphql::Result<AuthPayload> {
        let state = ctx.data_unchecked::<AppState>();
        state.login_user(&email, &password).await
    }

    async fn create_post(
        &self,
        ctx: &Context<'_>,
        content: String,
        media: Option<Vec<String>>,
        parent: Option<String>,
    ) -> async_graphql::Result<Post> {
        let state = ctx.data_unchecked::<AppState>();
        state.create_post("user", &content, media.unwrap_or_default(), parent).await
    }

    async fn delete_post(&self, ctx: &Context<'_>, cid: String) -> async_graphql::Result<bool> {
        let state = ctx.data_unchecked::<AppState>();
        state.delete_post(&cid).await
    }

    async fn follow(
        &self,
        ctx: &Context<'_>,
        peer_id: String,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data_unchecked::<AppState>();
        state.follow_user("user", &peer_id).await
    }

    async fn unfollow(
        &self,
        ctx: &Context<'_>,
        peer_id: String,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data_unchecked::<AppState>();
        state.unfollow_user("user", &peer_id).await
    }

    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_cid: Option<String>,
    ) -> async_graphql::Result<Profile> {
        let state = ctx.data_unchecked::<AppState>();
        state.update_profile("user", display_name, bio, avatar_cid).await
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn new_posts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)] _following: bool,
    ) -> impl Stream<Item = Post> {
        let state = ctx.data_unchecked::<AppState>();
        let mut rx = state.event_tx.subscribe();
        async_stream::stream! {
            while let Ok(event) = rx.recv().await {
                let GatewayEvent::NewPost(post) = event;
                yield post;
            }
        }
    }
}

#[derive(Clone)]
pub enum GatewayEvent {
    NewPost(Post),
}
