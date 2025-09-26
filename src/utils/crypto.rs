use crate::utils::error::{ErpError, ErpResult};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::{thread_rng, RngCore};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone)]
pub struct HashConfig {
    pub bcrypt_cost: u32,
}

impl Default for HashConfig {
    fn default() -> Self {
        Self {
            bcrypt_cost: DEFAULT_COST,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HashedPassword {
    hash: String,
}

impl HashedPassword {
    pub fn new(hash: String) -> Self {
        Self { hash }
    }

    pub fn as_str(&self) -> &str {
        &self.hash
    }

    pub fn verify(&self, password: &str) -> ErpResult<bool> {
        verify(password, &self.hash)
            .map_err(|e| ErpError::internal(format!("Password verification failed: {}", e)))
    }
}

impl fmt::Display for HashedPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hash)
    }
}

impl From<String> for HashedPassword {
    fn from(hash: String) -> Self {
        Self { hash }
    }
}

pub fn hash_password(password: &str, config: Option<&HashConfig>) -> ErpResult<HashedPassword> {
    let cost = config.map(|c| c.bcrypt_cost).unwrap_or(DEFAULT_COST);

    hash(password, cost)
        .map(HashedPassword::new)
        .map_err(|e| ErpError::internal(format!("Password hashing failed: {}", e)))
}

pub fn verify_password(password: &str, hashed_password: &HashedPassword) -> ErpResult<bool> {
    hashed_password.verify(password)
}

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn hash_string(data: &str) -> String {
    hash_data(data.as_bytes())
}

pub fn secure_compare(a: &str, b: &str) -> bool {
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

pub fn generate_token() -> String {
    let mut token = [0u8; 32];
    thread_rng().fill_bytes(&mut token);
    hex::encode(token)
}

pub fn generate_secure_id() -> String {
    let mut id = [0u8; 16];
    thread_rng().fill_bytes(&mut id);
    hex::encode(id)
}

#[derive(Debug, Clone)]
pub struct ApiKey {
    key: String,
    hash: String,
}

impl ApiKey {
    pub fn generate() -> Self {
        let key = generate_token();
        let hash = hash_string(&key);
        Self { key, hash }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn verify(&self, key: &str) -> bool {
        secure_compare(&hash_string(key), &self.hash)
    }

    pub fn from_hash(hash: String) -> Self {
        Self {
            key: String::new(),
            hash,
        }
    }
}

pub fn mask_sensitive_data(data: &str, visible_chars: usize) -> String {
    if data.len() <= visible_chars {
        "*".repeat(data.len())
    } else {
        let visible = &data[..visible_chars];
        let masked = "*".repeat(data.len() - visible_chars);
        format!("{}{}", visible, masked)
    }
}

pub fn validate_checksum(data: &str, expected_checksum: &str) -> ErpResult<()> {
    let calculated_checksum = hash_string(data);

    if secure_compare(&calculated_checksum, expected_checksum) {
        Ok(())
    } else {
        Err(ErpError::validation(
            "checksum",
            "does not match expected value",
        ))
    }
}

#[derive(Debug, Clone)]
pub struct HashingService {
    config: HashConfig,
}

impl HashingService {
    pub fn new() -> Self {
        Self {
            config: HashConfig::default(),
        }
    }

    pub fn with_config(config: HashConfig) -> Self {
        Self { config }
    }

    pub fn hash_password(&self, password: &str) -> ErpResult<HashedPassword> {
        hash_password(password, Some(&self.config))
    }

    pub fn verify_password(&self, password: &str, hashed_password: &str) -> ErpResult<bool> {
        let hashed = HashedPassword::new(hashed_password.to_string());
        verify_password(password, &hashed)
    }

    pub fn generate_salt(&self) -> [u8; 32] {
        generate_salt()
    }

    pub fn hash_data(&self, data: &[u8]) -> String {
        hash_data(data)
    }

    pub fn hash_string(&self, data: &str) -> String {
        hash_string(data)
    }
}

impl Default for HashingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let config = HashConfig::default();

        let hashed = hash_password(password, Some(&config)).unwrap();
        assert!(verify_password(password, &hashed).unwrap());
        assert!(!verify_password("wrong_password", &hashed).unwrap());
    }

    #[test]
    fn test_hash_data() {
        let data = b"hello world";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_secure_compare() {
        assert!(secure_compare("hello", "hello"));
        assert!(!secure_compare("hello", "world"));
        assert!(!secure_compare("hello", "hello world"));
    }

    #[test]
    fn test_generate_token() {
        let token1 = generate_token();
        let token2 = generate_token();

        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_api_key() {
        let api_key = ApiKey::generate();

        assert!(api_key.verify(api_key.key()));
        assert!(!api_key.verify("wrong_key"));
    }

    #[test]
    fn test_mask_sensitive_data() {
        assert_eq!(mask_sensitive_data("password123", 4), "pass*******");
        assert_eq!(mask_sensitive_data("short", 4), "*****");
        assert_eq!(mask_sensitive_data("test", 6), "****");
    }

    #[test]
    fn test_validate_checksum() {
        let data = "test data";
        let checksum = hash_string(data);

        assert!(validate_checksum(data, &checksum).is_ok());
        assert!(validate_checksum(data, "wrong_checksum").is_err());
    }

    #[test]
    fn test_hashed_password() {
        let password = "test123";
        let hashed = hash_password(password, None).unwrap();

        assert!(hashed.verify(password).unwrap());
        assert!(!hashed.verify("wrong").unwrap());

        let hash_str = hashed.to_string();
        assert!(!hash_str.is_empty());

        let from_string = HashedPassword::from(hash_str.clone());
        assert_eq!(from_string.as_str(), hash_str);
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 32);
    }

    #[test]
    fn test_hash_string() {
        let data = "test string";
        let hash1 = hash_string(data);
        let hash2 = hash_string(data);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash_string("different string"));
    }

    #[test]
    fn test_generate_secure_id() {
        let id1 = generate_secure_id();
        let id2 = generate_secure_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 32); // 16 bytes = 32 hex chars
        assert_eq!(id2.len(), 32);
    }

    #[test]
    fn test_api_key_from_hash() {
        let api_key = ApiKey::generate();
        let hash = api_key.hash().to_string();

        let api_key_from_hash = ApiKey::from_hash(hash.clone());
        assert_eq!(api_key_from_hash.hash(), hash);
        assert_eq!(api_key_from_hash.key(), ""); // Key should be empty
    }

    #[test]
    fn test_hash_config() {
        let config = HashConfig { bcrypt_cost: 6 };
        let password = "testpass";

        let hashed = hash_password(password, Some(&config)).unwrap();
        assert!(hashed.verify(password).unwrap());
    }

    #[test]
    fn test_hashing_service() {
        let service = HashingService::new();
        let password = "service_test_password";

        let hashed_pass = service.hash_password(password).unwrap();
        assert!(service
            .verify_password(password, hashed_pass.as_str())
            .unwrap());
        assert!(!service
            .verify_password("wrong", hashed_pass.as_str())
            .unwrap());

        let salt = service.generate_salt();
        assert_eq!(salt.len(), 32);

        let hash = service.hash_string("test data");
        assert!(!hash.is_empty());

        let data_hash = service.hash_data(b"binary data");
        assert_eq!(data_hash, hash_data(b"binary data"));
    }

    #[test]
    fn test_hashing_service_with_config() {
        let config = HashConfig { bcrypt_cost: 4 };
        let service = HashingService::with_config(config);

        let password = "config_test";
        let hashed = service.hash_password(password).unwrap();
        assert!(service.verify_password(password, hashed.as_str()).unwrap());
    }

    #[test]
    fn test_edge_cases() {
        // Test empty string
        let empty_hash = hash_string("");
        assert!(!empty_hash.is_empty());

        // Test mask with zero visible chars
        assert_eq!(mask_sensitive_data("secret", 0), "******");

        // Test secure compare with empty strings
        assert!(secure_compare("", ""));

        // Test API key verification with empty key
        let api_key = ApiKey::from_hash("some_hash".to_string());
        assert!(!api_key.verify("any_key"));
    }
}
