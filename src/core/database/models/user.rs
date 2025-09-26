use crate::utils::crypto::HashedPassword;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Manager,
    Employee,
    Viewer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

impl User {
    pub fn new(request: CreateUserRequest, password_hash: HashedPassword) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            username: request.username,
            email: request.email,
            password_hash: password_hash.to_string(),
            first_name: request.first_name,
            last_name: request.last_name,
            role: request.role,
            status: UserStatus::Pending,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    pub fn can_login(&self) -> bool {
        self.is_active() && !self.is_locked()
    }

    pub fn update(&mut self, request: UpdateUserRequest) {
        if let Some(username) = request.username {
            self.username = username;
        }
        if let Some(email) = request.email {
            self.email = email;
        }
        if let Some(first_name) = request.first_name {
            self.first_name = first_name;
        }
        if let Some(last_name) = request.last_name {
            self.last_name = last_name;
        }
        if let Some(role) = request.role {
            self.role = role;
        }
        if let Some(status) = request.status {
            self.status = status;
        }
        self.updated_at = Utc::now();
    }

    pub fn record_login_attempt(&mut self, success: bool) {
        if success {
            self.failed_login_attempts = 0;
            self.locked_until = None;
            self.last_login_at = Some(Utc::now());
        } else {
            self.failed_login_attempts += 1;

            if self.failed_login_attempts >= 5 {
                self.locked_until = Some(Utc::now() + chrono::Duration::minutes(15));
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn unlock(&mut self) {
        self.locked_until = None;
        self.failed_login_attempts = 0;
        self.updated_at = Utc::now();
    }

    pub fn change_password(&mut self, new_password_hash: HashedPassword) {
        self.password_hash = new_password_hash.to_string();
        self.updated_at = Utc::now();
    }

    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            role: self.role.clone(),
            status: self.status.clone(),
            last_login_at: self.last_login_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl UserRole {
    pub fn all() -> Vec<Self> {
        vec![Self::Admin, Self::Manager, Self::Employee, Self::Viewer]
    }

    pub fn permissions(&self) -> Vec<&'static str> {
        match self {
            Self::Admin => vec![
                "users:create",
                "users:read",
                "users:update",
                "users:delete",
                "products:create",
                "products:read",
                "products:update",
                "products:delete",
                "customers:create",
                "customers:read",
                "customers:update",
                "customers:delete",
                "orders:create",
                "orders:read",
                "orders:update",
                "orders:delete",
                "reports:read",
                "config:read",
                "config:update",
            ],
            Self::Manager => vec![
                "users:read",
                "products:create",
                "products:read",
                "products:update",
                "products:delete",
                "customers:create",
                "customers:read",
                "customers:update",
                "customers:delete",
                "orders:create",
                "orders:read",
                "orders:update",
                "orders:delete",
                "reports:read",
            ],
            Self::Employee => vec![
                "products:read",
                "products:update",
                "customers:create",
                "customers:read",
                "customers:update",
                "orders:create",
                "orders:read",
                "orders:update",
            ],
            Self::Viewer => vec![
                "products:read",
                "customers:read",
                "orders:read",
                "reports:read",
            ],
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions().contains(&permission)
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_manage_config(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_create_orders(&self) -> bool {
        matches!(self, Self::Admin | Self::Manager | Self::Employee)
    }

    pub fn can_delete_orders(&self) -> bool {
        matches!(self, Self::Admin | Self::Manager)
    }
}

impl UserStatus {
    pub fn all() -> Vec<Self> {
        vec![Self::Active, Self::Inactive, Self::Suspended, Self::Pending]
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn can_login(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Manager => write!(f, "manager"),
            Self::Employee => write!(f, "employee"),
            Self::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Suspended => write!(f, "suspended"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "manager" => Ok(Self::Manager),
            "employee" => Ok(Self::Employee),
            "viewer" => Ok(Self::Viewer),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "suspended" => Ok(Self::Suspended),
            "pending" => Ok(Self::Pending),
            _ => Err(format!("Invalid user status: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::crypto::HashedPassword;

    #[test]
    fn test_user_creation() {
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::Employee,
        };

        let password_hash = HashedPassword::new("$2b$12$test_hash".to_string());
        let user = User::new(request, password_hash);

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Employee);
        assert_eq!(user.status, UserStatus::Pending);
        assert_eq!(user.failed_login_attempts, 0);
        assert!(user.locked_until.is_none());
    }

    #[test]
    fn test_user_full_name() {
        let mut user = User {
            id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            role: UserRole::Employee,
            status: UserStatus::Active,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(user.full_name(), "John Doe");
    }

    #[test]
    fn test_user_login_attempts() {
        let mut user = User {
            id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::Employee,
            status: UserStatus::Active,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        user.record_login_attempt(false);
        assert_eq!(user.failed_login_attempts, 1);

        for _ in 0..4 {
            user.record_login_attempt(false);
        }
        assert_eq!(user.failed_login_attempts, 5);
        assert!(user.locked_until.is_some());
        assert!(user.is_locked());

        user.record_login_attempt(true);
        assert_eq!(user.failed_login_attempts, 0);
        assert!(user.locked_until.is_none());
        assert!(!user.is_locked());
    }

    #[test]
    fn test_user_role_permissions() {
        let admin = UserRole::Admin;
        let employee = UserRole::Employee;
        let viewer = UserRole::Viewer;

        assert!(admin.has_permission("users:create"));
        assert!(!employee.has_permission("users:create"));
        assert!(!viewer.has_permission("users:create"));

        assert!(admin.can_manage_users());
        assert!(!employee.can_manage_users());

        assert!(admin.can_create_orders());
        assert!(employee.can_create_orders());
        assert!(!viewer.can_create_orders());
    }

    #[test]
    fn test_user_status_methods() {
        let active = UserStatus::Active;
        let inactive = UserStatus::Inactive;

        assert!(active.is_active());
        assert!(!inactive.is_active());

        assert!(active.can_login());
        assert!(!inactive.can_login());
    }
}
