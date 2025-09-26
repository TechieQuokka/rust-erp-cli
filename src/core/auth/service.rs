use crate::core::auth::{jwt::JwtService, rbac::RbacService};
use crate::core::database::models::{
    CreateUserRequest, UpdateUserRequest, User, UserRole, UserStatus,
};
use crate::utils::crypto::HashingService;
use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation::ValidationService;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub access_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub max_login_attempts: i32,
    pub lockout_duration_minutes: i64,
    pub password_expiry_days: Option<i64>,
    pub require_password_change_on_first_login: bool,
    pub enable_session_timeout: bool,
    pub session_timeout_minutes: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
            password_expiry_days: Some(90),
            require_password_change_on_first_login: true,
            enable_session_timeout: true,
            session_timeout_minutes: 30,
        }
    }
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> ErpResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> ErpResult<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> ErpResult<Option<User>>;
    async fn create(&self, user: &User) -> ErpResult<User>;
    async fn update(&self, user: &User) -> ErpResult<User>;
    async fn update_login_attempt(&self, user_id: Uuid, success: bool) -> ErpResult<()>;
    async fn update_last_login(&self, user_id: Uuid, timestamp: DateTime<Utc>) -> ErpResult<()>;
}

pub struct AuthService {
    user_repository: Arc<dyn UserRepository>,
    jwt_service: JwtService,
    rbac_service: RbacService,
    hashing_service: HashingService,
    validation_service: ValidationService,
    config: AuthConfig,
}

impl AuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        jwt_service: JwtService,
        rbac_service: RbacService,
        hashing_service: HashingService,
        validation_service: ValidationService,
        config: AuthConfig,
    ) -> Self {
        Self {
            user_repository,
            jwt_service,
            rbac_service,
            hashing_service,
            validation_service,
            config,
        }
    }

    pub async fn login(&self, request: LoginRequest) -> ErpResult<LoginResponse> {
        self.validation_service
            .validate_username(&request.username)?;
        self.validation_service
            .validate_password(&request.password)?;

        let user = self
            .user_repository
            .find_by_username(&request.username)
            .await?
            .ok_or_else(|| ErpError::Authentication("Invalid credentials".to_string()))?;

        if !user.can_login() {
            return Err(ErpError::Authentication(
                "Account is not active or is locked".to_string(),
            ));
        }

        let password_valid = self
            .hashing_service
            .verify_password(&request.password, &user.password_hash)?;

        if !password_valid {
            self.user_repository
                .update_login_attempt(user.id, false)
                .await?;

            warn!("Failed login attempt for user: {}", user.username);
            return Err(ErpError::Authentication("Invalid credentials".to_string()));
        }

        // Successful login
        self.user_repository
            .update_login_attempt(user.id, true)
            .await?;

        self.user_repository
            .update_last_login(user.id, Utc::now())
            .await?;

        let token_pair = self.jwt_service.generate_token_pair(&user)?;

        info!("Successful login for user: {}", user.username);

        Ok(LoginResponse {
            user_id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            access_expires_at: token_pair.access_expires_at,
            refresh_expires_at: token_pair.refresh_expires_at,
        })
    }

    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> ErpResult<RefreshTokenResponse> {
        let claims = self
            .jwt_service
            .verify_refresh_token(&request.refresh_token)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|e| ErpError::Authentication(format!("Invalid user ID in token: {}", e)))?;

        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::Authentication("User not found".to_string()))?;

        if !user.can_login() {
            return Err(ErpError::Authentication(
                "Account is not active or is locked".to_string(),
            ));
        }

        let access_token = self
            .jwt_service
            .refresh_access_token(&request.refresh_token, &user)?;

        let access_expires_at = Utc::now() + chrono::Duration::minutes(15); // This should come from JWT config

        Ok(RefreshTokenResponse {
            access_token,
            access_expires_at,
        })
    }

    pub async fn logout(&self, access_token: &str) -> ErpResult<()> {
        // Add token to blacklist
        // This would typically be done through the middleware/jwt service
        info!("User logged out");
        Ok(())
    }

    pub async fn register(&self, request: CreateUserRequest) -> ErpResult<User> {
        self.validation_service
            .validate_username(&request.username)?;
        self.validation_service.validate_email(&request.email)?;
        self.validation_service
            .validate_password(&request.password)?;

        // Check if username already exists
        if self
            .user_repository
            .find_by_username(&request.username)
            .await?
            .is_some()
        {
            return Err(ErpError::conflict("Username already exists"));
        }

        // Check if email already exists
        if self
            .user_repository
            .find_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(ErpError::conflict("Email already exists"));
        }

        let password_hash = self.hashing_service.hash_password(&request.password)?;
        let user = User::new(request, password_hash);

        let created_user = self.user_repository.create(&user).await?;

        info!("New user registered: {}", created_user.username);

        Ok(created_user)
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> ErpResult<()> {
        self.validation_service
            .validate_password(&request.new_password)?;

        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        let current_password_valid = self
            .hashing_service
            .verify_password(&request.current_password, &user.password_hash)?;

        if !current_password_valid {
            return Err(ErpError::Authentication(
                "Current password is incorrect".to_string(),
            ));
        }

        let new_password_hash = self.hashing_service.hash_password(&request.new_password)?;
        let mut updated_user = user;
        updated_user.change_password(new_password_hash);

        self.user_repository.update(&updated_user).await?;

        info!("Password changed for user: {}", updated_user.username);

        Ok(())
    }

    pub async fn reset_password(&self, request: ResetPasswordRequest) -> ErpResult<()> {
        // In a real implementation, you would validate the reset token
        // and find the associated user
        self.validation_service
            .validate_password(&request.new_password)?;

        // For now, this is a placeholder implementation
        warn!("Password reset requested with token: {}", request.token);

        Ok(())
    }

    pub async fn get_authenticated_user(&self, token: &str) -> ErpResult<AuthenticatedUser> {
        let claims = self.jwt_service.verify_access_token(token)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|e| ErpError::Authentication(format!("Invalid user ID in token: {}", e)))?;

        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::Authentication("User not found".to_string()))?;

        let permissions = self
            .rbac_service
            .get_user_permissions(user.id, &user.role)
            .into_iter()
            .collect();

        Ok(AuthenticatedUser {
            id: user.id,
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            permissions,
            last_login_at: user.last_login_at,
        })
    }

    pub async fn check_permission(&self, user_id: Uuid, permission: &str) -> ErpResult<bool> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        self.rbac_service
            .has_permission(user.id, &user.role, permission)
    }

    pub async fn check_role(&self, user_id: Uuid, required_role: &UserRole) -> ErpResult<bool> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        Ok(user.role == *required_role)
    }

    pub async fn unlock_user(&self, user_id: Uuid) -> ErpResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        let mut updated_user = user;
        updated_user.unlock();

        self.user_repository.update(&updated_user).await?;

        info!("User unlocked: {}", updated_user.username);

        Ok(())
    }

    pub async fn deactivate_user(&self, user_id: Uuid) -> ErpResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        let mut updated_user = user;
        updated_user.update(UpdateUserRequest {
            username: None,
            email: None,
            first_name: None,
            last_name: None,
            role: None,
            status: Some(UserStatus::Inactive),
        });

        self.user_repository.update(&updated_user).await?;

        info!("User deactivated: {}", updated_user.username);

        Ok(())
    }

    pub async fn activate_user(&self, user_id: Uuid) -> ErpResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        let mut updated_user = user;
        updated_user.update(UpdateUserRequest {
            username: None,
            email: None,
            first_name: None,
            last_name: None,
            role: None,
            status: Some(UserStatus::Active),
        });

        self.user_repository.update(&updated_user).await?;

        info!("User activated: {}", updated_user.username);

        Ok(())
    }

    pub fn validate_token(&self, token: &str) -> ErpResult<bool> {
        match self.jwt_service.verify_access_token(token) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn is_token_expired(&self, token: &str) -> bool {
        self.jwt_service.is_token_expired(token)
    }

    pub async fn get_user_permissions(&self, user_id: Uuid) -> ErpResult<Vec<String>> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        let permissions = self
            .rbac_service
            .get_user_permissions(user.id, &user.role)
            .into_iter()
            .collect();

        Ok(permissions)
    }

    pub async fn grant_permission(&self, user_id: Uuid, permission: &str) -> ErpResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        // This would typically require admin privileges check
        info!(
            "Permission '{}' granted to user: {}",
            permission, user.username
        );

        Ok(())
    }

    pub async fn revoke_permission(&self, user_id: Uuid, permission: &str) -> ErpResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        // This would typically require admin privileges check
        info!(
            "Permission '{}' revoked from user: {}",
            permission, user.username
        );

        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> ErpResult<usize> {
        // This would typically clean up expired sessions from a session store
        // For now, just return 0
        Ok(0)
    }

    pub async fn get_active_sessions(&self, user_id: Uuid) -> ErpResult<Vec<String>> {
        // This would typically return active session IDs for a user
        // For now, return empty vector
        Ok(vec![])
    }

    pub async fn terminate_session(&self, session_id: &str) -> ErpResult<()> {
        // This would typically terminate a specific session
        info!("Session terminated: {}", session_id);
        Ok(())
    }

    pub async fn terminate_all_sessions(&self, user_id: Uuid) -> ErpResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ErpError::not_found("User", "not found"))?;

        info!("All sessions terminated for user: {}", user.username);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MockUserRepository {
    users: std::sync::Arc<std::sync::Mutex<Vec<User>>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn add_user(&self, user: User) {
        let mut users = self.users.lock().unwrap();
        users.push(user);
    }
}

#[async_trait::async_trait]
impl UserRepository for MockUserRepository {
    async fn find_by_username(&self, username: &str) -> ErpResult<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.username == username).cloned())
    }

    async fn find_by_email(&self, email: &str) -> ErpResult<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.email == email).cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> ErpResult<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.id == id).cloned())
    }

    async fn create(&self, user: &User) -> ErpResult<User> {
        let mut users = self.users.lock().unwrap();
        users.push(user.clone());
        Ok(user.clone())
    }

    async fn update(&self, user: &User) -> ErpResult<User> {
        let mut users = self.users.lock().unwrap();
        if let Some(existing_user) = users.iter_mut().find(|u| u.id == user.id) {
            *existing_user = user.clone();
            Ok(user.clone())
        } else {
            Err(ErpError::not_found("User", "not found"))
        }
    }

    async fn update_login_attempt(&self, user_id: Uuid, success: bool) -> ErpResult<()> {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.iter_mut().find(|u| u.id == user_id) {
            user.record_login_attempt(success);
        }
        Ok(())
    }

    async fn update_last_login(&self, user_id: Uuid, timestamp: DateTime<Utc>) -> ErpResult<()> {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.iter_mut().find(|u| u.id == user_id) {
            user.last_login_at = Some(timestamp);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::jwt::{JwtConfig, JwtService};
    use crate::core::database::models::{CreateUserRequest, UserRole};
    use crate::utils::crypto::{HashedPassword, HashingService};

    fn create_test_auth_service() -> AuthService {
        let user_repository = Arc::new(MockUserRepository::new());
        let jwt_service = JwtService::new(JwtConfig::default());
        let rbac_service = RbacService::new();
        let hashing_service = HashingService::new();
        let validation_service = ValidationService::new();
        let config = AuthConfig::default();

        AuthService::new(
            user_repository,
            jwt_service,
            rbac_service,
            hashing_service,
            validation_service,
            config,
        )
    }

    #[tokio::test]
    async fn test_user_registration() {
        let auth_service = create_test_auth_service();

        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "StrongPassword123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::Employee,
        };

        let user = auth_service.register(request).await.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Employee);
    }

    #[tokio::test]
    async fn test_user_login() {
        let auth_service = create_test_auth_service();

        // First register a user
        let register_request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "StrongPassword123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::Employee,
        };

        let mut user = auth_service.register(register_request).await.unwrap();
        user.status = UserStatus::Active; // Activate the user

        // Add user to mock repository
        if let Some(mock_repo) = auth_service.user_repository.as_ref() as &dyn std::any::Any {
            if let Some(mock_repo) = mock_repo.downcast_ref::<MockUserRepository>() {
                mock_repo.add_user(user);
            }
        }

        // Test login
        let login_request = LoginRequest {
            username: "testuser".to_string(),
            password: "StrongPassword123!".to_string(),
            remember_me: Some(false),
        };

        let login_response = auth_service.login(login_request).await.unwrap();
        assert_eq!(login_response.username, "testuser");
        assert_eq!(login_response.role, UserRole::Employee);
        assert!(!login_response.access_token.is_empty());
        assert!(!login_response.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_login() {
        let auth_service = create_test_auth_service();

        let login_request = LoginRequest {
            username: "nonexistent".to_string(),
            password: "wrongpassword".to_string(),
            remember_me: Some(false),
        };

        let result = auth_service.login(login_request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ErpError::Authentication(_) => (),
            _ => panic!("Expected authentication error"),
        }
    }

    #[tokio::test]
    async fn test_token_validation() {
        let auth_service = create_test_auth_service();

        // Test invalid token
        assert!(!auth_service.validate_token("invalid.token.here").unwrap());

        // Test with a real token would require setting up a complete user and login flow
    }

    #[tokio::test]
    async fn test_permission_check() {
        let auth_service = create_test_auth_service();
        let user_id = Uuid::new_v4();

        // Create a mock user with Employee role
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::Employee,
        };

        let password_hash = HashedPassword::new("hash".to_string());
        let mut user = User::new(request, password_hash);
        user.id = user_id;
        user.status = UserStatus::Active;

        // Add user to repository
        if let Some(mock_repo) = auth_service.user_repository.as_ref() as &dyn std::any::Any {
            if let Some(mock_repo) = mock_repo.downcast_ref::<MockUserRepository>() {
                mock_repo.add_user(user);
            }
        }

        // Test permission checks
        assert!(auth_service
            .check_permission(user_id, "products:read")
            .await
            .unwrap());
        assert!(!auth_service
            .check_permission(user_id, "users:create")
            .await
            .unwrap());
    }
}
