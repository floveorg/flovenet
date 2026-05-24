pub mod auth;
pub mod schema;

use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{Request, Response, Schema};
use axum::{
    Router,
    extract::Extension,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::get,
};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::schema::{AuthPayload, GatewayEvent, MutationRoot, Post, Profile, QueryRoot, SubscriptionRoot};
use crypto::generate_keypair;
use identity::PeerId;

pub type GatewaySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(Clone, Default)]
pub struct InMemoryStore {
    profiles: Arc<RwLock<HashMap<String, Profile>>>,
    posts: Arc<RwLock<HashMap<String, Post>>>,
    followers: Arc<RwLock<HashMap<String, Vec<String>>>>,
    following: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl InMemoryStore {
    pub async fn save_profile(&self, peer_id: &str, profile: Profile) {
        let mut profiles = self.profiles.write().await;
        profiles.insert(peer_id.to_string(), profile);
    }

    pub async fn get_profile(&self, peer_id: &str) -> Option<Profile> {
        let profiles = self.profiles.read().await;
        profiles.get(peer_id).cloned()
    }

    pub async fn get_profiles(&self) -> Vec<Profile> {
        let profiles = self.profiles.read().await;
        profiles.values().cloned().collect()
    }

    pub async fn save_post(&self, cid: &str, post: Post) {
        let mut posts = self.posts.write().await;
        posts.insert(cid.to_string(), post);
    }

    pub async fn get_post(&self, cid: &str) -> Option<Post> {
        let posts = self.posts.read().await;
        posts.get(cid).cloned()
    }

    pub async fn get_posts(&self) -> Vec<Post> {
        let posts = self.posts.read().await;
        let mut all: Vec<Post> = posts.values().cloned().collect();
        all.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        all
    }

    pub async fn delete_post(&self, cid: &str) -> bool {
        let mut posts = self.posts.write().await;
        posts.remove(cid).is_some()
    }

    pub async fn follow(&self, follower: &str, followee: &str) {
        {
            let mut follow = self.followers.write().await;
            follow.entry(followee.to_string()).or_default().push(follower.to_string());
        }
        {
            let mut follow = self.following.write().await;
            follow.entry(follower.to_string()).or_default().push(followee.to_string());
        }
    }

    pub async fn unfollow(&self, follower: &str, followee: &str) {
        {
            let mut follow = self.followers.write().await;
            if let Some(f) = follow.get_mut(followee) {
                f.retain(|x| x != follower);
            }
        }
        {
            let mut follow = self.following.write().await;
            if let Some(f) = follow.get_mut(follower) {
                f.retain(|x| x != followee);
            }
        }
    }

    pub async fn get_followers(&self, peer_id: &str) -> Vec<String> {
        let follow = self.followers.read().await;
        follow.get(peer_id).cloned().unwrap_or_default()
    }

    pub async fn get_following(&self, peer_id: &str) -> Vec<String> {
        let follow = self.following.read().await;
        follow.get(peer_id).cloned().unwrap_or_default()
    }
}

pub struct AppState {
    pub auth: auth::AuthManager,
    pub event_tx: broadcast::Sender<GatewayEvent>,
    pub store: InMemoryStore,
}

impl AppState {
    pub async fn get_profile(&self, peer_id: &str) -> Option<Profile> {
        self.store.get_profile(peer_id).await
    }

    pub async fn get_post(&self, cid: &str) -> Option<Post> {
        self.store.get_post(cid).await
    }

    pub async fn get_feed(&self, limit: usize, offset: usize) -> Vec<schema::FeedItem> {
        let posts = self.store.get_posts().await;
        let mut items = Vec::with_capacity(limit);
        for post in posts.into_iter().skip(offset).take(limit) {
            let author = self
                .store
                .get_profile(&post.author)
                .await
                .unwrap_or_else(|| Profile {
                    peer_id: post.author.clone(),
                    display_name: post.author.clone(),
                    bio: None,
                    avatar_cid: None,
                    follower_count: 0,
                    following_count: 0,
                    post_count: 0,
                    reputation: None,
                });
            items.push(schema::FeedItem { post, author, score: None });
        }
        items
    }

    pub async fn search_profiles(&self, query: &str) -> Vec<Profile> {
        let profiles = self.store.get_profiles().await;
        let q = query.to_lowercase();
        profiles
            .into_iter()
            .filter(|p| p.display_name.to_lowercase().contains(&q) || p.peer_id.to_lowercase().contains(&q))
            .collect()
    }

    pub async fn search_posts(&self, query: &str) -> Vec<Post> {
        let posts = self.store.get_posts().await;
        let q = query.to_lowercase();
        posts
            .into_iter()
            .filter(|p| p.content.to_lowercase().contains(&q))
            .collect()
    }

    pub async fn get_followers(&self, peer_id: &str, _limit: usize) -> Vec<Profile> {
        let peer_ids = self.store.get_followers(peer_id).await;
        let mut profiles = Vec::new();
        for pid in peer_ids {
            if let Some(p) = self.store.get_profile(&pid).await {
                profiles.push(p);
            }
        }
        profiles
    }

    pub async fn get_following(&self, peer_id: &str, _limit: usize) -> Vec<Profile> {
        let peer_ids = self.store.get_following(peer_id).await;
        let mut profiles = Vec::new();
        for pid in peer_ids {
            if let Some(p) = self.store.get_profile(&pid).await {
                profiles.push(p);
            }
        }
        profiles
    }

    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> async_graphql::Result<AuthPayload> {
        let kp = generate_keypair();
        let pk_bytes = kp.verifying_key().to_bytes().to_vec();
        let token = self
            .auth
            .register(email, password, display_name, pk_bytes.clone())
            .await
            .map_err(async_graphql::Error::new)?;

        let peer_id = PeerId::from_public_key_bytes(&pk_bytes);
        let profile = Profile {
            peer_id: peer_id.0.clone(),
            display_name: display_name.to_string(),
            bio: None,
            avatar_cid: None,
            follower_count: 0,
            following_count: 0,
            post_count: 0,
            reputation: None,
        };
        self.store.save_profile(&peer_id.0, profile.clone()).await;
        Ok(AuthPayload { token, profile })
    }

    pub async fn login_user(
        &self,
        email: &str,
        password: &str,
    ) -> async_graphql::Result<AuthPayload> {
        let (token, user) = self
            .auth
            .login(email, password)
            .await
            .map_err(async_graphql::Error::new)?;

        let peer_id = PeerId::from_public_key_bytes(&user.public_key);
        let profile = self
            .store
            .get_profile(&peer_id.0)
            .await
            .unwrap_or_else(|| Profile {
                peer_id: peer_id.0,
                display_name: user.display_name.clone(),
                bio: None,
                avatar_cid: None,
                follower_count: 0,
                following_count: 0,
                post_count: 0,
                reputation: None,
            });
        Ok(AuthPayload { token, profile })
    }

    pub async fn create_post(
        &self,
        user_id: &str,
        content: &str,
        media: Vec<String>,
        parent: Option<String>,
    ) -> async_graphql::Result<Post> {
        let cid = format!("post-{}", Uuid::new_v4());
        let post = Post {
            cid: cid.clone(),
            author: user_id.to_string(),
            content: content.to_string(),
            media,
            parent,
            reply_count: 0,
            like_count: 0,
            timestamp: chrono::Utc::now(),
            signature: Vec::new(),
        };
        self.store.save_post(&cid, post.clone()).await;
        let _ = self.event_tx.send(GatewayEvent::NewPost(post.clone()));
        Ok(post)
    }

    pub async fn delete_post(&self, cid: &str) -> async_graphql::Result<bool> {
        Ok(self.store.delete_post(cid).await)
    }

    pub async fn follow_user(
        &self,
        follower_id: &str,
        followee_id: &str,
    ) -> async_graphql::Result<bool> {
        self.store.follow(follower_id, followee_id).await;
        Ok(true)
    }

    pub async fn unfollow_user(
        &self,
        follower_id: &str,
        followee_id: &str,
    ) -> async_graphql::Result<bool> {
        self.store.unfollow(follower_id, followee_id).await;
        Ok(true)
    }

    pub async fn update_profile(
        &self,
        user_id: &str,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_cid: Option<String>,
    ) -> async_graphql::Result<Profile> {
        let mut profile = self
            .store
            .get_profile(user_id)
            .await
            .ok_or_else(|| async_graphql::Error::new("profile not found"))?;
        if let Some(name) = display_name {
            profile.display_name = name;
        }
        if let Some(b) = bio {
            profile.bio = Some(b);
        }
        if let Some(cid) = avatar_cid {
            profile.avatar_cid = Some(cid);
        }
        self.store.save_profile(user_id, profile.clone()).await;
        Ok(profile)
    }
}

pub fn build_schema(state: AppState) -> GatewaySchema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(state)
        .finish()
}

pub async fn run_gateway(config: GatewayConfig, state: AppState) -> anyhow::Result<()> {
    let schema = build_schema(state);

    let app = Router::new()
        .route("/graphql", get(playground).post(graphql_handler))
        .layer(Extension(schema))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([Method::GET, Method::POST]),
        );

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("GraphQL gateway at http://{addr}/graphql");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn playground() -> impl IntoResponse {
    axum::response::Html(
        async_graphql::http::playground_source(
            async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
        ),
    )
}

async fn graphql_handler(
    Extension(schema): Extension<GatewaySchema>,
    body: String,
) -> impl IntoResponse {
    let req: Request = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::response::Json(serde_json::json!({"error": format!("{e}")})),
            )
        }
    };
    let resp: Response = schema.execute(req).await;
    let status = if resp.is_err() {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::OK
    };
    (status, axum::response::Json(serde_json::to_value(&resp).unwrap_or_default()))
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub port: u16,
    pub peer_id: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            peer_id: String::new(),
        }
    }
}
