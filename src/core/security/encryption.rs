use crate::utils::error::{ErpError, ErpResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub master_key: Option<Vec<u8>>,
    pub key_derivation_iterations: u32,
    pub nonce_size: usize,
    pub tag_size: usize,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            master_key: None,
            key_derivation_iterations: 100_000,
            nonce_size: 12,
            tag_size: 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub salt: Option<String>,
    pub algorithm: String,
}

impl EncryptedData {
    pub fn new(ciphertext: String, nonce: String, algorithm: String) -> Self {
        Self {
            ciphertext,
            nonce,
            salt: None,
            algorithm,
        }
    }

    pub fn with_salt(mut self, salt: String) -> Self {
        self.salt = Some(salt);
        self
    }
}

impl fmt::Display for EncryptedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptedData[{}]", self.algorithm)
    }
}

pub struct EncryptionService {
    config: EncryptionConfig,
    cipher: Option<Aes256Gcm>,
}

impl EncryptionService {
    pub fn new(config: EncryptionConfig) -> ErpResult<Self> {
        let cipher = if let Some(ref key) = config.master_key {
            if key.len() != 32 {
                return Err(ErpError::validation("encryption_key", "must be 32 bytes"));
            }
            Some(
                Aes256Gcm::new_from_slice(key)
                    .map_err(|e| ErpError::internal(format!("Failed to create cipher: {}", e)))?,
            )
        } else {
            None
        };

        Ok(Self { config, cipher })
    }

    pub fn with_default_config() -> ErpResult<Self> {
        Self::new(EncryptionConfig::default())
    }

    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn set_master_key(&mut self, key: Vec<u8>) -> ErpResult<()> {
        if key.len() != 32 {
            return Err(ErpError::validation("encryption_key", "must be 32 bytes"));
        }

        self.config.master_key = Some(key.clone());
        self.cipher = Some(
            Aes256Gcm::new_from_slice(&key)
                .map_err(|e| ErpError::internal(format!("Failed to create cipher: {}", e)))?,
        );

        debug!("Master encryption key updated");
        Ok(())
    }

    pub fn encrypt(&self, plaintext: &str) -> ErpResult<EncryptedData> {
        let cipher = self
            .cipher
            .as_ref()
            .ok_or_else(|| ErpError::internal("Encryption not configured - no master key"))?;

        // Generate random nonce
        let mut nonce_bytes = vec![0u8; self.config.nonce_size];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| ErpError::internal(format!("Encryption failed: {}", e)))?;

        // Encode to base64
        let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);
        let nonce_b64 = general_purpose::STANDARD.encode(&nonce_bytes);

        debug!("Data encrypted successfully");

        Ok(EncryptedData::new(
            ciphertext_b64,
            nonce_b64,
            "AES-256-GCM".to_string(),
        ))
    }

    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> ErpResult<String> {
        let cipher = self
            .cipher
            .as_ref()
            .ok_or_else(|| ErpError::internal("Encryption not configured - no master key"))?;

        // Decode from base64
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted_data.ciphertext)
            .map_err(|e| ErpError::validation("ciphertext", &format!("Invalid base64: {}", e)))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .map_err(|e| ErpError::validation("nonce", &format!("Invalid base64: {}", e)))?;

        if nonce_bytes.len() != self.config.nonce_size {
            return Err(ErpError::validation("nonce", "Invalid nonce size"));
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| ErpError::Authentication(format!("Decryption failed: {}", e)))?;

        let result = String::from_utf8(plaintext)
            .map_err(|e| ErpError::internal(format!("Invalid UTF-8: {}", e)))?;

        debug!("Data decrypted successfully");
        Ok(result)
    }

    pub fn encrypt_with_password(
        &self,
        plaintext: &str,
        password: &str,
    ) -> ErpResult<EncryptedData> {
        // Generate salt for key derivation
        let salt = argon2::password_hash::SaltString::generate(&mut OsRng);

        // Derive key from password
        let key = self.derive_key_from_password(password, salt.as_str())?;

        // Create cipher with derived key
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| ErpError::internal(format!("Failed to create cipher: {}", e)))?;

        // Generate random nonce
        let mut nonce_bytes = vec![0u8; self.config.nonce_size];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| ErpError::internal(format!("Encryption failed: {}", e)))?;

        // Encode to base64
        let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);
        let nonce_b64 = general_purpose::STANDARD.encode(&nonce_bytes);
        let salt_b64 = general_purpose::STANDARD.encode(salt.as_str().as_bytes());

        debug!("Data encrypted with password successfully");

        Ok(
            EncryptedData::new(ciphertext_b64, nonce_b64, "AES-256-GCM-PBKDF2".to_string())
                .with_salt(salt_b64),
        )
    }

    pub fn decrypt_with_password(
        &self,
        encrypted_data: &EncryptedData,
        password: &str,
    ) -> ErpResult<String> {
        let salt_b64 = encrypted_data.salt.as_ref().ok_or_else(|| {
            ErpError::validation("salt", "Missing salt for password-based decryption")
        })?;

        let salt_bytes = general_purpose::STANDARD
            .decode(salt_b64)
            .map_err(|e| ErpError::validation("salt", &format!("Invalid base64: {}", e)))?;

        let salt_str = std::str::from_utf8(&salt_bytes)
            .map_err(|e| ErpError::validation("salt", &format!("Invalid UTF-8: {}", e)))?;

        // Derive key from password
        let key = self.derive_key_from_password(password, salt_str)?;

        // Create cipher with derived key
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| ErpError::internal(format!("Failed to create cipher: {}", e)))?;

        // Decode from base64
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted_data.ciphertext)
            .map_err(|e| ErpError::validation("ciphertext", &format!("Invalid base64: {}", e)))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .map_err(|e| ErpError::validation("nonce", &format!("Invalid base64: {}", e)))?;

        if nonce_bytes.len() != self.config.nonce_size {
            return Err(ErpError::validation("nonce", "Invalid nonce size"));
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| ErpError::Authentication(format!("Decryption failed: {}", e)))?;

        let result = String::from_utf8(plaintext)
            .map_err(|e| ErpError::internal(format!("Invalid UTF-8: {}", e)))?;

        debug!("Data decrypted with password successfully");
        Ok(result)
    }

    pub fn hash_password(&self, password: &str) -> ErpResult<String> {
        let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| ErpError::internal(format!("Password hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> ErpResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| ErpError::validation("password_hash", &format!("Invalid hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn derive_key_from_password(&self, password: &str, salt: &str) -> ErpResult<Vec<u8>> {
        let salt_string = argon2::password_hash::SaltString::from_b64(salt)
            .map_err(|e| ErpError::validation("salt", &format!("Invalid salt: {}", e)))?;

        let argon2 = Argon2::default();

        let mut key = vec![0u8; 32];
        argon2
            .hash_password_into(
                password.as_bytes(),
                salt_string.as_str().as_bytes(),
                &mut key,
            )
            .map_err(|e| ErpError::internal(format!("Key derivation failed: {}", e)))?;

        Ok(key)
    }

    pub fn secure_compare(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();

        let mut result = 0u8;
        for i in 0..a_bytes.len() {
            result |= a_bytes[i] ^ b_bytes[i];
        }

        result == 0
    }

    pub fn generate_secure_token(&self, length: usize) -> String {
        let mut token = vec![0u8; length];
        OsRng.fill_bytes(&mut token);
        general_purpose::URL_SAFE_NO_PAD.encode(&token)
    }

    pub fn mask_sensitive_data(&self, data: &str, visible_chars: usize) -> String {
        if data.len() <= visible_chars {
            "*".repeat(data.len())
        } else {
            let visible = &data[..visible_chars];
            let masked = "*".repeat(data.len() - visible_chars);
            format!("{}{}", visible, masked)
        }
    }

    pub fn is_configured(&self) -> bool {
        self.cipher.is_some()
    }
}

// Field-level encryption helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField<T> {
    encrypted_data: EncryptedData,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T> EncryptedField<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn encrypt(service: &EncryptionService, value: &T) -> ErpResult<Self> {
        let json = serde_json::to_string(value)
            .map_err(|e| ErpError::internal(format!("Serialization failed: {}", e)))?;

        let encrypted_data = service.encrypt(&json)?;

        Ok(Self {
            encrypted_data,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn decrypt(&self, service: &EncryptionService) -> ErpResult<T> {
        let json = service.decrypt(&self.encrypted_data)?;

        let value = serde_json::from_str(&json)
            .map_err(|e| ErpError::internal(format!("Deserialization failed: {}", e)))?;

        Ok(value)
    }

    pub fn encrypt_with_password(
        service: &EncryptionService,
        value: &T,
        password: &str,
    ) -> ErpResult<Self> {
        let json = serde_json::to_string(value)
            .map_err(|e| ErpError::internal(format!("Serialization failed: {}", e)))?;

        let encrypted_data = service.encrypt_with_password(&json, password)?;

        Ok(Self {
            encrypted_data,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn decrypt_with_password(
        &self,
        service: &EncryptionService,
        password: &str,
    ) -> ErpResult<T> {
        let json = service.decrypt_with_password(&self.encrypted_data, password)?;

        let value = serde_json::from_str(&json)
            .map_err(|e| ErpError::internal(format!("Deserialization failed: {}", e)))?;

        Ok(value)
    }
}

// Utility functions for common encryption patterns
pub fn encrypt_pii(service: &EncryptionService, pii_data: &str) -> ErpResult<EncryptedData> {
    if pii_data.is_empty() {
        return Err(ErpError::validation("pii_data", "cannot be empty"));
    }

    service.encrypt(pii_data)
}

pub fn decrypt_pii(
    service: &EncryptionService,
    encrypted_pii: &EncryptedData,
) -> ErpResult<String> {
    service.decrypt(encrypted_pii)
}

pub fn encrypt_financial_data(
    service: &EncryptionService,
    amount: &str,
    currency: &str,
) -> ErpResult<EncryptedData> {
    let financial_data = serde_json::json!({
        "amount": amount,
        "currency": currency,
        "timestamp": chrono::Utc::now()
    });

    let json_str = serde_json::to_string(&financial_data)
        .map_err(|e| ErpError::internal(format!("Failed to serialize financial data: {}", e)))?;

    service.encrypt(&json_str)
}

// Key rotation utilities
pub struct KeyRotationManager {
    current_service: EncryptionService,
    previous_service: Option<EncryptionService>,
    rotation_id: String,
}

impl KeyRotationManager {
    pub fn new(current_key: Vec<u8>) -> ErpResult<Self> {
        let config = EncryptionConfig {
            master_key: Some(current_key),
            ..EncryptionConfig::default()
        };

        let current_service = EncryptionService::new(config)?;

        Ok(Self {
            current_service,
            previous_service: None,
            rotation_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    pub fn rotate_key(&mut self, new_key: Vec<u8>) -> ErpResult<()> {
        let new_config = EncryptionConfig {
            master_key: Some(new_key),
            ..EncryptionConfig::default()
        };

        let new_service = EncryptionService::new(new_config)?;

        // Keep the current service as previous
        self.previous_service = Some(std::mem::replace(&mut self.current_service, new_service));
        self.rotation_id = uuid::Uuid::new_v4().to_string();

        debug!("Key rotation completed with ID: {}", self.rotation_id);
        Ok(())
    }

    pub fn encrypt(&self, plaintext: &str) -> ErpResult<EncryptedData> {
        self.current_service.encrypt(plaintext)
    }

    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> ErpResult<String> {
        // Try current key first
        match self.current_service.decrypt(encrypted_data) {
            Ok(result) => Ok(result),
            Err(_) => {
                // Try previous key if available
                if let Some(ref previous) = self.previous_service {
                    warn!("Attempting decryption with previous key");
                    previous.decrypt(encrypted_data)
                } else {
                    Err(ErpError::Authentication(
                        "Unable to decrypt data with available keys".to_string(),
                    ))
                }
            }
        }
    }

    pub fn re_encrypt(&self, encrypted_data: &EncryptedData) -> ErpResult<EncryptedData> {
        // Decrypt with old key and encrypt with new key
        let plaintext = self.decrypt(encrypted_data)?;
        self.encrypt(&plaintext)
    }

    pub fn get_rotation_id(&self) -> &str {
        &self.rotation_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_service_creation() {
        let key = EncryptionService::generate_key();
        assert_eq!(key.len(), 32);

        let config = EncryptionConfig {
            master_key: Some(key),
            ..EncryptionConfig::default()
        };

        let service = EncryptionService::new(config);
        assert!(service.is_ok());
        assert!(service.unwrap().is_configured());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionService::generate_key();
        let config = EncryptionConfig {
            master_key: Some(key),
            ..EncryptionConfig::default()
        };

        let service = EncryptionService::new(config).unwrap();

        let plaintext = "Hello, World!";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_eq!(encrypted.algorithm, "AES-256-GCM");
    }

    #[test]
    fn test_password_based_encryption() {
        let service = EncryptionService::with_default_config().unwrap();

        let plaintext = "Secret message";
        let password = "strong_password_123";

        let encrypted = service.encrypt_with_password(plaintext, password).unwrap();
        let decrypted = service.decrypt_with_password(&encrypted, password).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_eq!(encrypted.algorithm, "AES-256-GCM-PBKDF2");
        assert!(encrypted.salt.is_some());

        // Wrong password should fail
        let wrong_result = service.decrypt_with_password(&encrypted, "wrong_password");
        assert!(wrong_result.is_err());
    }

    #[test]
    fn test_password_hashing() {
        let service = EncryptionService::with_default_config().unwrap();

        let password = "test_password_123";
        let hash = service.hash_password(password).unwrap();

        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("wrong_password", &hash).unwrap());
        assert!(!hash.is_empty());
        assert_ne!(hash, password);
    }

    #[test]
    fn test_secure_compare() {
        let service = EncryptionService::with_default_config().unwrap();

        assert!(service.secure_compare("hello", "hello"));
        assert!(!service.secure_compare("hello", "world"));
        assert!(!service.secure_compare("hello", "hello world"));
        assert!(service.secure_compare("", ""));
    }

    #[test]
    fn test_secure_token_generation() {
        let service = EncryptionService::with_default_config().unwrap();

        let token1 = service.generate_secure_token(32);
        let token2 = service.generate_secure_token(32);

        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());
    }

    #[test]
    fn test_mask_sensitive_data() {
        let service = EncryptionService::with_default_config().unwrap();

        assert_eq!(service.mask_sensitive_data("password123", 4), "pass*******");
        assert_eq!(service.mask_sensitive_data("short", 4), "shor*");
        assert_eq!(service.mask_sensitive_data("test", 6), "****");
    }

    #[test]
    fn test_encrypted_field() {
        let key = EncryptionService::generate_key();
        let config = EncryptionConfig {
            master_key: Some(key),
            ..EncryptionConfig::default()
        };

        let service = EncryptionService::new(config).unwrap();

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let original = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let encrypted_field = EncryptedField::encrypt(&service, &original).unwrap();
        let decrypted = encrypted_field.decrypt(&service).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_pii_encryption() {
        let key = EncryptionService::generate_key();
        let config = EncryptionConfig {
            master_key: Some(key),
            ..EncryptionConfig::default()
        };

        let service = EncryptionService::new(config).unwrap();

        let pii = "123-45-6789";
        let encrypted = encrypt_pii(&service, pii).unwrap();
        let decrypted = decrypt_pii(&service, &encrypted).unwrap();

        assert_eq!(pii, decrypted);

        // Empty PII should fail
        let empty_result = encrypt_pii(&service, "");
        assert!(empty_result.is_err());
    }

    #[test]
    fn test_financial_data_encryption() {
        let key = EncryptionService::generate_key();
        let config = EncryptionConfig {
            master_key: Some(key),
            ..EncryptionConfig::default()
        };

        let service = EncryptionService::new(config).unwrap();

        let encrypted = encrypt_financial_data(&service, "100.50", "USD").unwrap();
        let decrypted_json = service.decrypt(&encrypted).unwrap();

        let financial_data: serde_json::Value = serde_json::from_str(&decrypted_json).unwrap();
        assert_eq!(financial_data["amount"], "100.50");
        assert_eq!(financial_data["currency"], "USD");
        assert!(financial_data["timestamp"].is_string());
    }

    #[test]
    fn test_key_rotation() {
        let old_key = EncryptionService::generate_key();
        let new_key = EncryptionService::generate_key();

        let mut rotation_manager = KeyRotationManager::new(old_key).unwrap();

        let plaintext = "sensitive data";
        let encrypted_with_old = rotation_manager.encrypt(plaintext).unwrap();

        // Rotate key
        rotation_manager.rotate_key(new_key).unwrap();

        // Should still be able to decrypt old data
        let decrypted_old = rotation_manager.decrypt(&encrypted_with_old).unwrap();
        assert_eq!(plaintext, decrypted_old);

        // New data should be encrypted with new key
        let encrypted_with_new = rotation_manager.encrypt(plaintext).unwrap();
        let decrypted_new = rotation_manager.decrypt(&encrypted_with_new).unwrap();
        assert_eq!(plaintext, decrypted_new);

        // Re-encrypt old data with new key
        let re_encrypted = rotation_manager.re_encrypt(&encrypted_with_old).unwrap();
        let decrypted_re = rotation_manager.decrypt(&re_encrypted).unwrap();
        assert_eq!(plaintext, decrypted_re);
    }

    #[test]
    fn test_invalid_configurations() {
        // Invalid key size
        let invalid_config = EncryptionConfig {
            master_key: Some(vec![0u8; 16]), // Wrong size
            ..EncryptionConfig::default()
        };

        let result = EncryptionService::new(invalid_config);
        assert!(result.is_err());

        // No key configured
        let service = EncryptionService::with_default_config().unwrap();
        assert!(!service.is_configured());

        let encrypt_result = service.encrypt("test");
        assert!(encrypt_result.is_err());
    }
}
