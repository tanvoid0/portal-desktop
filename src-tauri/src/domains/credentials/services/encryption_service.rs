/**
 * Encryption Service - Handles credential encryption/decryption
 */

use ring::aead::{Aad, LessSafeKey, UnboundKey, AES_256_GCM, Nonce};
use ring::rand::{SecureRandom, SystemRandom};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose};
use super::super::CredentialError;

#[derive(Debug, Clone)]
pub struct EncryptionService {
    rng: Arc<SystemRandom>,
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {
            rng: Arc::new(SystemRandom::new()),
        }
    }

    /// Encrypt a plaintext value
    pub fn encrypt(&self, plaintext: &str, key: &[u8]) -> Result<EncryptionResult, CredentialError> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| CredentialError::EncryptionFailed(e.to_string()))?;

        let nonce_bytes = self.generate_nonce()?;
        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|e| CredentialError::EncryptionFailed(e.to_string()))?;

        let less_safe_key = LessSafeKey::new(unbound_key);
        let mut ciphertext = plaintext.as_bytes().to_vec();
        let tag = less_safe_key.seal_in_place_separate_tag(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|e| CredentialError::EncryptionFailed(e.to_string()))?;

        ciphertext.extend_from_slice(tag.as_ref());

        Ok(EncryptionResult {
            encrypted: general_purpose::STANDARD.encode(&ciphertext),
            iv: general_purpose::STANDARD.encode(&nonce_bytes),
            tag: general_purpose::STANDARD.encode(tag.as_ref()),
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// Decrypt an encrypted value
    pub fn decrypt(&self, request: DecryptionRequest) -> Result<String, CredentialError> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, request.key.as_bytes())
            .map_err(|e| CredentialError::DecryptionFailed(e.to_string()))?;

        let nonce_bytes = general_purpose::STANDARD.decode(&request.iv)
            .map_err(|e| CredentialError::DecryptionFailed(e.to_string()))?;
        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|e| CredentialError::DecryptionFailed(e.to_string()))?;

        let less_safe_key = LessSafeKey::new(unbound_key);
        let mut ciphertext = general_purpose::STANDARD.decode(&request.encrypted)
            .map_err(|e| CredentialError::DecryptionFailed(e.to_string()))?;

        let plaintext = less_safe_key.open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|e| CredentialError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext.to_vec())
            .map_err(|e| CredentialError::DecryptionFailed(e.to_string()))
    }

    /// Generate a random nonce
    fn generate_nonce(&self) -> Result<[u8; 12], CredentialError> {
        let mut nonce = [0u8; 12];
        self.rng.fill(&mut nonce)
            .map_err(|e| CredentialError::EncryptionFailed(e.to_string()))?;
        Ok(nonce)
    }

    /// Derive key from password using PBKDF2
    pub fn derive_key(&self, password: &str, salt: &[u8], iterations: u32) -> Result<[u8; 32], CredentialError> {
        use ring::pbkdf2;
        

        let mut key = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(iterations).unwrap(),
            salt,
            password.as_bytes(),
            &mut key,
        );

        Ok(key)
    }

    /// Generate a random salt
    pub fn generate_salt(&self) -> Result<[u8; 32], CredentialError> {
        let mut salt = [0u8; 32];
        self.rng.fill(&mut salt)
            .map_err(|e| CredentialError::EncryptionFailed(e.to_string()))?;
        Ok(salt)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionResult {
    pub encrypted: String,
    pub iv: String,
    pub tag: String,
    pub algorithm: String,
}

#[derive(Debug, Clone)]
pub struct DecryptionRequest {
    pub encrypted: String,
    pub iv: String,
    pub tag: String,
    pub algorithm: String,
    pub key: String,
}
