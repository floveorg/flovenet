use argon2::Argon2;
use chrono::{Duration, Utc};
use jsonwebtoken::{Header, Validation, decode, encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: Vec<u8>,
    pub public_key: Vec<u8>,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct AuthManager {
    jwt_secret: String,
    users: Arc<RwLock<HashMap<String, StoredUser>>>,
}

impl AuthManager {
    pub fn new(jwt_secret: impl Into<String>) -> Self {
        AuthManager {
            jwt_secret: jwt_secret.into(),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
        public_key: Vec<u8>,
    ) -> Result<String, String> {
        let mut users = self.users.write().await;
        if users.contains_key(email) {
            return Err("email already registered".into());
        }

        let id = uuid::Uuid::new_v4().to_string();
        let salt: Vec<u8> = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let mut hash = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut hash)
            .map_err(|e| format!("hashing error: {e}"))?;

        let user = StoredUser {
            id,
            email: email.to_string(),
            password_hash: hex::encode(hash),
            password_salt: salt,
            public_key,
            display_name: display_name.to_string(),
            created_at: Utc::now(),
        };

        let token = self.generate_token(&user)?;
        users.insert(email.to_string(), user);
        Ok(token)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(String, StoredUser), String> {
        let users = self.users.read().await;
        let user = users
            .get(email)
            .ok_or_else(|| "invalid email or password".to_string())?;

        let mut hash = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &user.password_salt, &mut hash)
            .map_err(|e| format!("hashing error: {e}"))?;

        let computed = hex::encode(hash);
        if computed != user.password_hash {
            return Err("invalid email or password".into());
        }

        let token = self.generate_token(user)?;
        Ok((token, user.clone()))
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let key = jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let token_data = decode::<Claims>(token, &key, &Validation::default())
            .map_err(|e| format!("invalid token: {e}"))?;
        Ok(token_data.claims)
    }

    fn generate_token(&self, user: &StoredUser) -> Result<String, String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::hours(24)).timestamp() as usize,
        };
        let key = jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &key)
            .map_err(|e| format!("token generation error: {e}"))
    }
}
