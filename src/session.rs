use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::prelude::*;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

const SESSION_TTL_SECS: u64 = 30 * 24 * 60 * 60; // 30 days
const NONCE_SIZE: usize = 12;

/// Represents a user session stored in Redis.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub email: String,
}

/// Encrypts data using AES-256-GCM with a random nonce.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Encryption(format!("Invalid key: {e}")))?;

    let nonce_bytes: [u8; NONCE_SIZE] = rand::random::<[u8; NONCE_SIZE]>();
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {e}")))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypts data that was encrypted with `encrypt`.
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, AppError> {
    if data.len() < NONCE_SIZE {
        return Err(AppError::Encryption("Data too short".into()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Encryption(format!("Invalid key: {e}")))?;

    let nonce_array: [u8; NONCE_SIZE] = data[..NONCE_SIZE]
        .try_into()
        .map_err(|_| AppError::Encryption("Invalid nonce".into()))?;
    let nonce = Nonce::from(nonce_array);
    let ciphertext = &data[NONCE_SIZE..];

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| AppError::Encryption(format!("Decryption failed: {e}")))
}

/// Decode a base64-encoded 32-byte encryption key.
pub fn decode_encryption_key(encoded: &str) -> Result<[u8; 32], AppError> {
    let bytes = BASE64_STANDARD
        .decode(encoded)
        .map_err(|e| AppError::Encryption(format!("Invalid base64 key: {e}")))?;

    bytes
        .try_into()
        .map_err(|_| AppError::Encryption("Encryption key must be exactly 32 bytes".into()))
}

/// Store a session in Redis.
pub async fn create_session(
    redis: &mut redis::aio::MultiplexedConnection,
    token: &str,
    session: &Session,
    key: &[u8; 32],
) -> Result<(), AppError> {
    let json = serde_json::to_vec(session)
        .map_err(|e| AppError::Serialization(format!("Session serialization failed: {e}")))?;

    let encrypted = encrypt(key, &json)?;
    let encoded = BASE64_STANDARD.encode(&encrypted);

    redis
        .set_ex::<_, _, ()>(format!("session:{token}"), encoded, SESSION_TTL_SECS)
        .await?;

    Ok(())
}

/// Retrieve a session from Redis.
pub async fn get_session(
    redis: &mut redis::aio::MultiplexedConnection,
    token: &str,
    key: &[u8; 32],
) -> Result<Session, AppError> {
    let encoded: Option<String> = redis.get(format!("session:{token}")).await?;

    let encoded = encoded.ok_or(AppError::SessionNotFound)?;

    let encrypted = BASE64_STANDARD
        .decode(&encoded)
        .map_err(|e| AppError::Encryption(format!("Invalid stored data: {e}")))?;

    let decrypted = decrypt(key, &encrypted)?;

    let session: Session = serde_json::from_slice(&decrypted)
        .map_err(|e| AppError::Serialization(format!("Session deserialization failed: {e}")))?;

    // Refresh TTL on access
    let _: () = redis
        .expire(format!("session:{token}"), SESSION_TTL_SECS as i64)
        .await?;

    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"hello, world!";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key1 = [42u8; 32];
        let key2 = [99u8; 32];
        let plaintext = b"secret data";

        let encrypted = encrypt(&key1, plaintext).unwrap();
        assert!(decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_too_short_data_fails() {
        let key = [42u8; 32];
        let short = vec![0u8; 5];

        assert!(decrypt(&key, &short).is_err());
    }

    #[test]
    fn test_decode_encryption_key_valid() {
        let key = [1u8; 32];
        let encoded = BASE64_STANDARD.encode(key);
        let decoded = decode_encryption_key(&encoded).unwrap();
        assert_eq!(decoded, key);
    }

    #[test]
    fn test_decode_encryption_key_wrong_size() {
        let key = [1u8; 16]; // too short
        let encoded = BASE64_STANDARD.encode(key);
        assert!(decode_encryption_key(&encoded).is_err());
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        let session = Session {
            email: "test@example.com".into(),
        };
        let json = serde_json::to_vec(&session).unwrap();
        let deserialized: Session = serde_json::from_slice(&json).unwrap();
        assert_eq!(deserialized.email, "test@example.com");
    }
}
