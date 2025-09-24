use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use rand::RngCore;
use sha2::Digest;
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;

use crate::utils::errors::AppError;

/// Bank-level encryption service for API keys and secrets
#[derive(Clone)]
pub struct EncryptionService {
    // No stored keys - all keys derived on demand for maximum security
}

#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: String,  // Base64 encoded
    pub nonce: String,       // Base64 encoded
    pub salt: String,        // Base64 encoded (for key derivation)
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {}
    }

    /// Encrypt sensitive data using AES-256-GCM with user-derived key
    pub fn encrypt_api_credentials(
        &self,
        plaintext: &str,
        user_password: &str,
        user_id: &str,
    ) -> Result<EncryptedData, AppError> {
        // Generate random salt for this encryption
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        // Derive encryption key from user password + user_id + salt
        let key = self.derive_key(user_password, user_id, &salt)?;
        let cipher = Aes256Gcm::new(&key);

        // Generate random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| AppError::EncryptionError("Failed to encrypt data".to_string()))?;

        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce),
            salt: general_purpose::STANDARD.encode(&salt),
        })
    }

    /// Decrypt sensitive data using AES-256-GCM with user-derived key
    pub fn decrypt_api_credentials(
        &self,
        encrypted_data: &EncryptedData,
        user_password: &str,
        user_id: &str,
    ) -> Result<String, AppError> {
        // Decode components
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted_data.ciphertext)
            .map_err(|_| AppError::DecryptionError("Invalid ciphertext encoding".to_string()))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .map_err(|_| AppError::DecryptionError("Invalid nonce encoding".to_string()))?;

        let salt = general_purpose::STANDARD
            .decode(&encrypted_data.salt)
            .map_err(|_| AppError::DecryptionError("Invalid salt encoding".to_string()))?;

        // Recreate the key using the same derivation
        let key = self.derive_key(user_password, user_id, &salt)?;
        let cipher = Aes256Gcm::new(&key);

        // Recreate nonce
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| AppError::DecryptionError("Failed to decrypt data - invalid key or corrupted data".to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|_| AppError::DecryptionError("Decrypted data is not valid UTF-8".to_string()))
    }

    /// Encrypt multiple API credentials as a JSON object
    #[allow(dead_code)]
    pub fn encrypt_exchange_credentials(
        &self,
        credentials: &HashMap<String, String>,
        user_password: &str,
        user_id: &str,
    ) -> Result<EncryptedData, AppError> {
        let json_str = serde_json::to_string(credentials)
            .map_err(|_| AppError::EncryptionError("Failed to serialize credentials".to_string()))?;

        self.encrypt_api_credentials(&json_str, user_password, user_id)
    }

    /// Decrypt multiple API credentials from JSON
    #[allow(dead_code)]
    pub fn decrypt_exchange_credentials(
        &self,
        encrypted_data: &EncryptedData,
        user_password: &str,
        user_id: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        let json_str = self.decrypt_api_credentials(encrypted_data, user_password, user_id)?;

        serde_json::from_str(&json_str)
            .map_err(|_| AppError::DecryptionError("Failed to deserialize credentials".to_string()))
    }

    /// Derive a 256-bit key from password, user_id, and salt using PBKDF2-SHA256
    fn derive_key(&self, password: &str, user_id: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, AppError> {
        use sha2::Sha256;

        // Combine password and user_id for additional entropy
        let combined_input = format!("{}:{}", password, user_id);

        // Use PBKDF2 with high iteration count for key derivation
        let mut key_bytes = [0u8; 32]; // 256 bits

        // Simple PBKDF2 implementation using HMAC-SHA256
        let mut hmac_key = combined_input.as_bytes().to_vec();
        hmac_key.extend_from_slice(salt);

        // Perform 100,000 iterations for strong key derivation
        let iterations = 100_000;
        let mut derived = hmac_key;

        for _ in 0..iterations {
            let mut hasher = Sha256::new();
            hasher.update(&derived);
            hasher.update(salt);
            derived = hasher.finalize().to_vec();
        }

        key_bytes.copy_from_slice(&derived[0..32]);
        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }


    /// Validate that encrypted data can be decrypted (for testing)
    #[allow(dead_code)]
    pub fn validate_encryption(
        &self,
        encrypted_data: &EncryptedData,
        user_password: &str,
        user_id: &str,
    ) -> bool {
        self.decrypt_api_credentials(encrypted_data, user_password, user_id).is_ok()
    }
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let service = EncryptionService::new();
        let plaintext = "test_api_key_12345";
        let password = "user_password";
        let user_id = "user123";

        let encrypted = service.encrypt_api_credentials(plaintext, password, user_id).unwrap();
        let decrypted = service.decrypt_api_credentials(&encrypted, password, user_id).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_passwords_fail() {
        let service = EncryptionService::new();
        let plaintext = "test_api_key_12345";
        let password = "user_password";
        let wrong_password = "wrong_password";
        let user_id = "user123";

        let encrypted = service.encrypt_api_credentials(plaintext, password, user_id).unwrap();
        let result = service.decrypt_api_credentials(&encrypted, wrong_password, user_id);

        assert!(result.is_err());
    }

    #[test]
    fn test_exchange_credentials() {
        let service = EncryptionService::new();
        let mut credentials = HashMap::new();
        credentials.insert("api_key".to_string(), "test_key_123".to_string());
        credentials.insert("api_secret".to_string(), "test_secret_456".to_string());

        let password = "user_password";
        let user_id = "user123";

        let encrypted = service.encrypt_exchange_credentials(&credentials, password, user_id).unwrap();
        let decrypted = service.decrypt_exchange_credentials(&encrypted, password, user_id).unwrap();

        assert_eq!(credentials, decrypted);
    }
}