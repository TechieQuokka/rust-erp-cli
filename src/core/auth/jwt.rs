use crate::core::database::models::{User, UserRole};
use crate::utils::error::{ErpError, ErpResult};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry: Duration,
    pub refresh_token_expiry: Duration,
    pub algorithm: Algorithm,
    pub issuer: String,
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-secret-key-change-this-in-production".to_string(),
            access_token_expiry: Duration::minutes(15),
            refresh_token_expiry: Duration::days(7),
            algorithm: Algorithm::HS256,
            issuer: "erp-cli".to_string(),
            audience: "erp-users".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,              // Subject (user ID)
    pub username: String,         // Username
    pub role: UserRole,           // User role
    pub permissions: Vec<String>, // User permissions
    pub iat: i64,                 // Issued at
    pub exp: i64,                 // Expiration time
    pub nbf: i64,                 // Not before
    pub iss: String,              // Issuer
    pub aud: String,              // Audience
    pub jti: String,              // JWT ID
    pub token_type: TokenType,    // Access or Refresh token
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("config", &self.config)
            .field("encoding_key", &"[HIDDEN]")
            .field("decoding_key", &"[HIDDEN]")
            .field("validation", &self.validation)
            .finish()
    }
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.secret.as_ref());

        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(std::slice::from_ref(&config.issuer));
        validation.set_audience(std::slice::from_ref(&config.audience));

        Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    pub fn generate_token_pair(&self, user: &User) -> ErpResult<TokenPair> {
        let now = Utc::now();
        let access_expires_at = now + self.config.access_token_expiry;
        let refresh_expires_at = now + self.config.refresh_token_expiry;

        let access_token = self.generate_access_token(user, now, access_expires_at)?;
        let refresh_token = self.generate_refresh_token(user, now, refresh_expires_at)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            access_expires_at,
            refresh_expires_at,
        })
    }

    pub fn generate_access_token(
        &self,
        user: &User,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> ErpResult<String> {
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.clone(),
            permissions: user
                .role
                .permissions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            iat: issued_at.timestamp(),
            exp: expires_at.timestamp(),
            nbf: issued_at.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        let header = Header::new(self.config.algorithm);
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| ErpError::Authentication(format!("Failed to encode access token: {}", e)))
    }

    pub fn generate_refresh_token(
        &self,
        user: &User,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> ErpResult<String> {
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.clone(),
            permissions: Vec::new(), // Refresh tokens don't need permissions
            iat: issued_at.timestamp(),
            exp: expires_at.timestamp(),
            nbf: issued_at.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        let header = Header::new(self.config.algorithm);
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| ErpError::Authentication(format!("Failed to encode refresh token: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> ErpResult<TokenData<Claims>> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| ErpError::Authentication(format!("Invalid token: {}", e)))
    }

    pub fn verify_access_token(&self, token: &str) -> ErpResult<Claims> {
        let token_data = self.verify_token(token)?;

        if token_data.claims.token_type != TokenType::Access {
            return Err(ErpError::Authentication(
                "Expected access token".to_string(),
            ));
        }

        Ok(token_data.claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> ErpResult<Claims> {
        let token_data = self.verify_token(token)?;

        if token_data.claims.token_type != TokenType::Refresh {
            return Err(ErpError::Authentication(
                "Expected refresh token".to_string(),
            ));
        }

        Ok(token_data.claims)
    }

    pub fn refresh_access_token(&self, refresh_token: &str, user: &User) -> ErpResult<String> {
        let claims = self.verify_refresh_token(refresh_token)?;

        if claims.sub != user.id.to_string() {
            return Err(ErpError::Authentication("Token user mismatch".to_string()));
        }

        let now = Utc::now();
        let expires_at = now + self.config.access_token_expiry;

        self.generate_access_token(user, now, expires_at)
    }

    pub fn extract_user_id(&self, token: &str) -> ErpResult<Uuid> {
        let claims = self.verify_access_token(token)?;
        Uuid::parse_str(&claims.sub)
            .map_err(|e| ErpError::Authentication(format!("Invalid user ID in token: {}", e)))
    }

    pub fn check_permission(&self, token: &str, required_permission: &str) -> ErpResult<bool> {
        let claims = self.verify_access_token(token)?;
        Ok(claims
            .permissions
            .contains(&required_permission.to_string()))
    }

    pub fn check_role(&self, token: &str, required_role: &UserRole) -> ErpResult<bool> {
        let claims = self.verify_access_token(token)?;
        Ok(claims.role == *required_role)
    }

    pub fn check_any_role(&self, token: &str, required_roles: &[UserRole]) -> ErpResult<bool> {
        let claims = self.verify_access_token(token)?;
        Ok(required_roles.contains(&claims.role))
    }

    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.verify_token(token) {
            Ok(_) => false,
            Err(ErpError::Authentication(msg)) => msg.contains("expired"),
            Err(_) => true,
        }
    }

    pub fn get_token_expiry(&self, token: &str) -> ErpResult<DateTime<Utc>> {
        let token_data = self.verify_token(token)?;
        let timestamp = token_data.claims.exp;
        DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| ErpError::Authentication("Invalid expiry timestamp".to_string()))
    }

    pub fn get_claims(&self, token: &str) -> ErpResult<Claims> {
        let token_data = self.verify_token(token)?;
        Ok(token_data.claims)
    }

    pub fn validate_token_format(token: &str) -> bool {
        // JWT tokens have 3 parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() == 3 && parts.iter().all(|part| !part.is_empty())
    }
}

#[derive(Debug)]
pub struct TokenBlacklist {
    blacklisted_tokens: HashSet<String>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            blacklisted_tokens: HashSet::new(),
        }
    }

    pub fn add_token(&mut self, jti: String) {
        self.blacklisted_tokens.insert(jti);
    }

    pub fn is_blacklisted(&self, jti: &str) -> bool {
        self.blacklisted_tokens.contains(jti)
    }

    pub fn remove_expired_tokens(&mut self, jwt_service: &JwtService) {
        let now = Utc::now();
        let mut to_remove = Vec::new();

        for jti in &self.blacklisted_tokens {
            if let Ok(claims) = jwt_service.verify_token(jti) {
                if claims.claims.exp < now.timestamp() {
                    to_remove.push(jti.clone());
                }
            } else {
                to_remove.push(jti.clone());
            }
        }

        for jti in to_remove {
            self.blacklisted_tokens.remove(&jti);
        }
    }

    pub fn clear(&mut self) {
        self.blacklisted_tokens.clear();
    }

    pub fn size(&self) -> usize {
        self.blacklisted_tokens.len()
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    jwt_service: JwtService,
    blacklist: std::sync::Arc<std::sync::Mutex<TokenBlacklist>>,
}

impl AuthMiddleware {
    pub fn new(jwt_service: JwtService) -> Self {
        Self {
            jwt_service,
            blacklist: std::sync::Arc::new(std::sync::Mutex::new(TokenBlacklist::new())),
        }
    }

    pub fn authenticate(&self, auth_header: &str) -> ErpResult<Claims> {
        let token = self.extract_bearer_token(auth_header)?;

        if !JwtService::validate_token_format(token) {
            return Err(ErpError::Authentication("Invalid token format".to_string()));
        }

        let claims = self.jwt_service.verify_access_token(token)?;

        // Check if token is blacklisted
        let blacklist = self.blacklist.lock().unwrap();
        if blacklist.is_blacklisted(&claims.jti) {
            return Err(ErpError::Authentication(
                "Token has been revoked".to_string(),
            ));
        }

        Ok(claims)
    }

    pub fn authorize(&self, auth_header: &str, required_permission: &str) -> ErpResult<Claims> {
        let claims = self.authenticate(auth_header)?;

        if !claims
            .permissions
            .contains(&required_permission.to_string())
        {
            return Err(ErpError::Authorization(format!(
                "Insufficient permissions. Required: {}",
                required_permission
            )));
        }

        Ok(claims)
    }

    pub fn authorize_role(&self, auth_header: &str, required_role: &UserRole) -> ErpResult<Claims> {
        let claims = self.authenticate(auth_header)?;

        if claims.role != *required_role {
            return Err(ErpError::Authorization(format!(
                "Insufficient role. Required: {}",
                required_role
            )));
        }

        Ok(claims)
    }

    pub fn authorize_any_role(
        &self,
        auth_header: &str,
        required_roles: &[UserRole],
    ) -> ErpResult<Claims> {
        let claims = self.authenticate(auth_header)?;

        if !required_roles.contains(&claims.role) {
            return Err(ErpError::Authorization(format!(
                "Insufficient role. Required one of: {:?}",
                required_roles
            )));
        }

        Ok(claims)
    }

    pub fn blacklist_token(&self, token: &str) -> ErpResult<()> {
        let claims = self.jwt_service.verify_token(token)?;
        let mut blacklist = self.blacklist.lock().unwrap();
        blacklist.add_token(claims.claims.jti);
        Ok(())
    }

    fn extract_bearer_token<'a>(&self, auth_header: &'a str) -> ErpResult<&'a str> {
        if !auth_header.starts_with("Bearer ") {
            return Err(ErpError::Authentication(
                "Authorization header must start with 'Bearer '".to_string(),
            ));
        }

        let token = &auth_header[7..]; // Remove "Bearer " prefix
        if token.is_empty() {
            return Err(ErpError::Authentication("Token is empty".to_string()));
        }

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::models::{CreateUserRequest, User, UserRole, UserStatus};
    use crate::utils::crypto::HashedPassword;

    fn create_test_user() -> User {
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::Employee,
        };

        let password_hash = HashedPassword::new("$2b$12$test_hash".to_string());
        let mut user = User::new(request, password_hash);
        user.status = UserStatus::Active;
        user
    }

    fn create_test_jwt_service() -> JwtService {
        let config = JwtConfig {
            secret: "test-secret-key-for-testing".to_string(),
            access_token_expiry: Duration::minutes(15),
            refresh_token_expiry: Duration::days(7),
            algorithm: Algorithm::HS256,
            issuer: "test-erp".to_string(),
            audience: "test-users".to_string(),
        };

        JwtService::new(config)
    }

    #[test]
    fn test_token_generation_and_verification() {
        let jwt_service = create_test_jwt_service();
        let user = create_test_user();

        let token_pair = jwt_service.generate_token_pair(&user).unwrap();

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert!(token_pair.access_expires_at > Utc::now());
        assert!(token_pair.refresh_expires_at > token_pair.access_expires_at);

        let claims = jwt_service
            .verify_access_token(&token_pair.access_token)
            .unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, user.role);
        assert_eq!(claims.token_type, TokenType::Access);

        let refresh_claims = jwt_service
            .verify_refresh_token(&token_pair.refresh_token)
            .unwrap();
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
        assert!(refresh_claims.permissions.is_empty());
    }

    #[test]
    fn test_token_refresh() {
        let jwt_service = create_test_jwt_service();
        let user = create_test_user();

        let token_pair = jwt_service.generate_token_pair(&user).unwrap();
        let new_access_token = jwt_service
            .refresh_access_token(&token_pair.refresh_token, &user)
            .unwrap();

        let claims = jwt_service.verify_access_token(&new_access_token).unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
    }

    #[test]
    fn test_permission_check() {
        let jwt_service = create_test_jwt_service();
        let user = create_test_user();

        let token_pair = jwt_service.generate_token_pair(&user).unwrap();

        assert!(jwt_service
            .check_permission(&token_pair.access_token, "products:read")
            .unwrap());
        assert!(!jwt_service
            .check_permission(&token_pair.access_token, "users:create")
            .unwrap());
    }

    #[test]
    fn test_role_check() {
        let jwt_service = create_test_jwt_service();
        let user = create_test_user();

        let token_pair = jwt_service.generate_token_pair(&user).unwrap();

        assert!(jwt_service
            .check_role(&token_pair.access_token, &UserRole::Employee)
            .unwrap());
        assert!(!jwt_service
            .check_role(&token_pair.access_token, &UserRole::Admin)
            .unwrap());

        assert!(jwt_service
            .check_any_role(
                &token_pair.access_token,
                &[UserRole::Employee, UserRole::Manager]
            )
            .unwrap());
        assert!(!jwt_service
            .check_any_role(
                &token_pair.access_token,
                &[UserRole::Admin, UserRole::Viewer]
            )
            .unwrap());
    }

    #[test]
    fn test_invalid_token() {
        let jwt_service = create_test_jwt_service();

        assert!(jwt_service
            .verify_access_token("invalid.token.here")
            .is_err());
        assert!(jwt_service
            .verify_refresh_token("invalid.token.here")
            .is_err());
        assert!(!JwtService::validate_token_format("invalid.token"));
        assert!(!JwtService::validate_token_format("invalid..token"));
    }

    #[test]
    fn test_auth_middleware() {
        let jwt_service = create_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service);
        let user = create_test_user();

        let token_pair = middleware.jwt_service.generate_token_pair(&user).unwrap();
        let auth_header = format!("Bearer {}", token_pair.access_token);

        let claims = middleware.authenticate(&auth_header).unwrap();
        assert_eq!(claims.sub, user.id.to_string());

        let claims = middleware.authorize(&auth_header, "products:read").unwrap();
        assert_eq!(claims.sub, user.id.to_string());

        assert!(middleware.authorize(&auth_header, "users:create").is_err());

        let claims = middleware
            .authorize_role(&auth_header, &UserRole::Employee)
            .unwrap();
        assert_eq!(claims.sub, user.id.to_string());

        assert!(middleware
            .authorize_role(&auth_header, &UserRole::Admin)
            .is_err());
    }

    #[test]
    fn test_token_blacklist() {
        let jwt_service = create_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service);
        let user = create_test_user();

        let token_pair = middleware.jwt_service.generate_token_pair(&user).unwrap();
        let auth_header = format!("Bearer {}", token_pair.access_token);

        assert!(middleware.authenticate(&auth_header).is_ok());

        middleware
            .blacklist_token(&token_pair.access_token)
            .unwrap();

        assert!(middleware.authenticate(&auth_header).is_err());
    }

    #[test]
    fn test_token_blacklist_cleanup() {
        let mut blacklist = TokenBlacklist::new();
        let jwt_service = create_test_jwt_service();

        blacklist.add_token("test-jti-1".to_string());
        blacklist.add_token("test-jti-2".to_string());

        assert_eq!(blacklist.size(), 2);
        assert!(blacklist.is_blacklisted("test-jti-1"));

        blacklist.remove_expired_tokens(&jwt_service);
        blacklist.clear();

        assert_eq!(blacklist.size(), 0);
        assert!(!blacklist.is_blacklisted("test-jti-1"));
    }
}
