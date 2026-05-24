use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed")]
    Encryption,

    #[error("decryption failed")]
    Decryption,

    #[error("signature verification failed")]
    SignatureVerification,

    #[error("key not found")]
    KeyNotFound,

    #[error("invalid key")]
    InvalidKey,

    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}
