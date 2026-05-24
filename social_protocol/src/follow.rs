use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follow {
    pub follower: String,
    pub followee: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signature: Vec<u8>,
}

impl Follow {
    pub fn to_signing_payload(&self) -> Vec<u8> {
        let payload = serde_json::json!({
            "follower": self.follower,
            "followee": self.followee,
            "timestamp": self.timestamp.to_rfc3339(),
        });
        serde_json::to_vec(&payload).unwrap_or_default()
    }

    pub fn sign_with(&mut self, signing_key: &ed25519_dalek::SigningKey) {
        use ed25519_dalek::Signer;
        let sig = signing_key.sign(&self.to_signing_payload());
        self.signature = sig.to_bytes().to_vec();
    }

    pub fn verify_signature(&self, public_key: &ed25519_dalek::VerifyingKey) -> bool {
        use ed25519_dalek::Verifier;
        let sig_bytes: [u8; 64] = match self.signature.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        public_key.verify(&self.to_signing_payload(), &sig).is_ok()
    }
}
