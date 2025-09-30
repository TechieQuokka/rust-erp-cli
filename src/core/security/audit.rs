use crate::utils::error::ErpResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditAction {
    Login,
    Logout,
    LoginFailed,
    UserCreated,
    UserUpdated,
    UserDeactivated,
    PasswordChanged,
    PermissionGranted,
    PermissionRevoked,
    DataAccessed,
    DataModified,
    DataDeleted,
    ConfigChanged,
    SystemError,
    SecurityViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub action: AuditAction,
    pub resource: Option<String>,
    pub resource_id: Option<String>,
    pub severity: AuditSeverity,
    pub details: HashMap<String, String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl AuditEvent {
    pub fn new(action: AuditAction, severity: AuditSeverity) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: None,
            username: None,
            action,
            resource: None,
            resource_id: None,
            severity,
            details: HashMap::new(),
            ip_address: None,
            user_agent: None,
            success: true,
            error_message: None,
        }
    }

    pub fn with_user(mut self, user_id: Uuid, username: String) -> Self {
        self.user_id = Some(user_id);
        self.username = Some(username);
        self
    }

    pub fn with_resource(mut self, resource: String, resource_id: Option<String>) -> Self {
        self.resource = Some(resource);
        self.resource_id = resource_id;
        self
    }

    pub fn with_client_info(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = details;
        self
    }

    pub fn add_detail<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    pub fn as_failure(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }
}

#[async_trait::async_trait]
pub trait AuditRepository: Send + Sync {
    async fn store_event(&self, event: &AuditEvent) -> ErpResult<()>;
    async fn get_events(&self, filters: AuditFilters) -> ErpResult<Vec<AuditEvent>>;
    async fn count_events(&self, filters: AuditFilters) -> ErpResult<u64>;
    async fn cleanup_old_events(&self, retention_days: i64) -> ErpResult<u64>;
}

#[derive(Debug, Clone, Default)]
pub struct AuditFilters {
    pub user_id: Option<Uuid>,
    pub action: Option<AuditAction>,
    pub severity: Option<AuditSeverity>,
    pub resource: Option<String>,
    pub success: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub struct AuditService {
    repository: Box<dyn AuditRepository>,
    config: AuditConfig,
}

#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enabled: bool,
    pub retention_days: i64,
    pub max_events_per_hour: u32,
    pub log_to_file: bool,
    pub log_failed_only: bool,
    pub sensitive_actions: Vec<AuditAction>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 90,
            max_events_per_hour: 1000,
            log_to_file: true,
            log_failed_only: false,
            sensitive_actions: vec![
                AuditAction::LoginFailed,
                AuditAction::PasswordChanged,
                AuditAction::PermissionGranted,
                AuditAction::PermissionRevoked,
                AuditAction::UserDeactivated,
                AuditAction::SecurityViolation,
                AuditAction::SystemError,
            ],
        }
    }
}

impl AuditService {
    pub fn new(repository: Box<dyn AuditRepository>, config: AuditConfig) -> Self {
        Self { repository, config }
    }

    pub async fn log_event(&self, event: AuditEvent) -> ErpResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if self.config.log_failed_only && event.success {
            return Ok(());
        }

        // Check rate limiting
        if !self.check_rate_limit(&event).await? {
            warn!(
                "Audit event rate limit exceeded for action: {:?}",
                event.action
            );
            return Ok(());
        }

        // Log to structured logging first
        self.log_to_tracing(&event);

        // Store in repository
        if let Err(e) = self.repository.store_event(&event).await {
            error!("Failed to store audit event: {}", e);
            return Err(e);
        }

        info!(
            "Audit event logged: {:?} for user: {:?}",
            event.action, event.username
        );
        Ok(())
    }

    pub async fn log_login_success(
        &self,
        user_id: Uuid,
        username: String,
        ip_address: Option<String>,
    ) -> ErpResult<()> {
        let event = AuditEvent::new(AuditAction::Login, AuditSeverity::Low)
            .with_user(user_id, username)
            .with_client_info(ip_address, None);

        self.log_event(event).await
    }

    pub async fn log_login_failure(
        &self,
        username: String,
        ip_address: Option<String>,
        reason: String,
    ) -> ErpResult<()> {
        let event = AuditEvent::new(AuditAction::LoginFailed, AuditSeverity::Medium)
            .with_client_info(ip_address, None)
            .add_detail("username", username)
            .add_detail("reason", reason)
            .as_failure("Login failed".to_string());

        self.log_event(event).await
    }

    pub async fn log_logout(&self, user_id: Uuid, username: String) -> ErpResult<()> {
        let event =
            AuditEvent::new(AuditAction::Logout, AuditSeverity::Low).with_user(user_id, username);

        self.log_event(event).await
    }

    pub async fn log_data_access(
        &self,
        user_id: Uuid,
        username: String,
        resource: String,
        resource_id: Option<String>,
    ) -> ErpResult<()> {
        let event = AuditEvent::new(AuditAction::DataAccessed, AuditSeverity::Low)
            .with_user(user_id, username)
            .with_resource(resource, resource_id);

        self.log_event(event).await
    }

    pub async fn log_data_modification(
        &self,
        user_id: Uuid,
        username: String,
        resource: String,
        resource_id: Option<String>,
        details: HashMap<String, String>,
    ) -> ErpResult<()> {
        let event = AuditEvent::new(AuditAction::DataModified, AuditSeverity::Medium)
            .with_user(user_id, username)
            .with_resource(resource, resource_id)
            .with_details(details);

        self.log_event(event).await
    }

    pub async fn log_security_violation(
        &self,
        user_id: Option<Uuid>,
        username: Option<String>,
        violation_type: String,
        details: HashMap<String, String>,
    ) -> ErpResult<()> {
        let mut event = AuditEvent::new(AuditAction::SecurityViolation, AuditSeverity::Critical)
            .with_details(details)
            .add_detail("violation_type", violation_type)
            .as_failure("Security violation detected".to_string());

        if let (Some(uid), Some(uname)) = (user_id, username) {
            event = event.with_user(uid, uname);
        }

        self.log_event(event).await
    }

    pub async fn log_permission_change(
        &self,
        admin_user_id: Uuid,
        admin_username: String,
        target_user_id: Uuid,
        permission: String,
        granted: bool,
    ) -> ErpResult<()> {
        let action = if granted {
            AuditAction::PermissionGranted
        } else {
            AuditAction::PermissionRevoked
        };

        let event = AuditEvent::new(action, AuditSeverity::High)
            .with_user(admin_user_id, admin_username)
            .add_detail("target_user_id", target_user_id.to_string())
            .add_detail("permission", permission)
            .add_detail("granted", granted.to_string());

        self.log_event(event).await
    }

    pub async fn get_user_activity(&self, user_id: Uuid, days: i64) -> ErpResult<Vec<AuditEvent>> {
        let filters = AuditFilters {
            user_id: Some(user_id),
            start_time: Some(Utc::now() - chrono::Duration::days(days)),
            end_time: Some(Utc::now()),
            limit: Some(1000),
            ..Default::default()
        };

        self.repository.get_events(filters).await
    }

    pub async fn get_security_events(&self, hours: i64) -> ErpResult<Vec<AuditEvent>> {
        let filters = AuditFilters {
            severity: Some(AuditSeverity::Critical),
            start_time: Some(Utc::now() - chrono::Duration::hours(hours)),
            end_time: Some(Utc::now()),
            limit: Some(500),
            ..Default::default()
        };

        self.repository.get_events(filters).await
    }

    pub async fn get_failed_logins(&self, hours: i64) -> ErpResult<Vec<AuditEvent>> {
        let filters = AuditFilters {
            action: Some(AuditAction::LoginFailed),
            start_time: Some(Utc::now() - chrono::Duration::hours(hours)),
            end_time: Some(Utc::now()),
            success: Some(false),
            limit: Some(1000),
            ..Default::default()
        };

        self.repository.get_events(filters).await
    }

    pub async fn cleanup_old_events(&self) -> ErpResult<u64> {
        self.repository
            .cleanup_old_events(self.config.retention_days)
            .await
    }

    async fn check_rate_limit(&self, _event: &AuditEvent) -> ErpResult<bool> {
        // Simple rate limiting - in production, use Redis or similar
        // For now, always allow
        Ok(true)
    }

    fn log_to_tracing(&self, event: &AuditEvent) {
        let fields = tracing::field::debug(&event);

        match event.severity {
            AuditSeverity::Critical => error!(fields, "AUDIT: {:?}", event.action),
            AuditSeverity::High => warn!(fields, "AUDIT: {:?}", event.action),
            AuditSeverity::Medium => info!(fields, "AUDIT: {:?}", event.action),
            AuditSeverity::Low => info!(fields, "AUDIT: {:?}", event.action),
        }
    }
}

// Mock implementation for testing
#[derive(Debug, Clone)]
pub struct MockAuditRepository {
    events: std::sync::Arc<std::sync::Mutex<Vec<AuditEvent>>>,
}

impl Default for MockAuditRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockAuditRepository {
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_all_events(&self) -> Vec<AuditEvent> {
        self.events.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl AuditRepository for MockAuditRepository {
    async fn store_event(&self, event: &AuditEvent) -> ErpResult<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(())
    }

    async fn get_events(&self, filters: AuditFilters) -> ErpResult<Vec<AuditEvent>> {
        let events = self.events.lock().unwrap();
        let mut filtered: Vec<AuditEvent> = events
            .iter()
            .filter(|e| {
                if let Some(user_id) = filters.user_id {
                    if e.user_id != Some(user_id) {
                        return false;
                    }
                }
                if let Some(ref action) = filters.action {
                    if std::mem::discriminant(&e.action) != std::mem::discriminant(action) {
                        return false;
                    }
                }
                if let Some(ref severity) = filters.severity {
                    if std::mem::discriminant(&e.severity) != std::mem::discriminant(severity) {
                        return false;
                    }
                }
                if let Some(success) = filters.success {
                    if e.success != success {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = filters.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    async fn count_events(&self, filters: AuditFilters) -> ErpResult<u64> {
        let events = self.get_events(filters).await?;
        Ok(events.len() as u64)
    }

    async fn cleanup_old_events(&self, retention_days: i64) -> ErpResult<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);
        let mut events = self.events.lock().unwrap();
        let original_count = events.len();
        events.retain(|e| e.timestamp > cutoff_date);
        let removed_count = original_count - events.len();
        Ok(removed_count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditAction::Login, AuditSeverity::Low)
            .with_user(Uuid::new_v4(), "testuser".to_string())
            .add_detail("ip", "127.0.0.1");

        assert!(event.success);
        assert_eq!(event.action, AuditAction::Login);
        assert_eq!(event.severity, AuditSeverity::Low);
        assert!(event.details.contains_key("ip"));
    }

    #[tokio::test]
    async fn test_audit_service_log_event() {
        let repository = Box::new(MockAuditRepository::new());
        let config = AuditConfig::default();
        let service = AuditService::new(repository, config);

        let event = AuditEvent::new(AuditAction::Login, AuditSeverity::Low)
            .with_user(Uuid::new_v4(), "testuser".to_string());

        let result = service.log_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_repository_filtering() {
        let repository = MockAuditRepository::new();
        let user_id = Uuid::new_v4();

        // Store multiple events
        let events = vec![
            AuditEvent::new(AuditAction::Login, AuditSeverity::Low)
                .with_user(user_id, "testuser".to_string()),
            AuditEvent::new(AuditAction::LoginFailed, AuditSeverity::Medium)
                .as_failure("Invalid password".to_string()),
            AuditEvent::new(AuditAction::Logout, AuditSeverity::Low)
                .with_user(user_id, "testuser".to_string()),
        ];

        for event in events {
            repository.store_event(&event).await.unwrap();
        }

        // Test user filtering
        let filters = AuditFilters {
            user_id: Some(user_id),
            ..Default::default()
        };
        let user_events = repository.get_events(filters).await.unwrap();
        assert_eq!(user_events.len(), 2); // Login and Logout

        // Test action filtering
        let filters = AuditFilters {
            action: Some(AuditAction::LoginFailed),
            ..Default::default()
        };
        let failed_events = repository.get_events(filters).await.unwrap();
        assert_eq!(failed_events.len(), 1);
    }

    #[tokio::test]
    async fn test_security_violation_logging() {
        let repository = Box::new(MockAuditRepository::new());
        let config = AuditConfig::default();
        let service = AuditService::new(repository, config);

        let mut details = HashMap::new();
        details.insert(
            "attempted_action".to_string(),
            "access_admin_panel".to_string(),
        );

        let result = service
            .log_security_violation(
                Some(Uuid::new_v4()),
                Some("hacker".to_string()),
                "unauthorized_access".to_string(),
                details,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_old_events() {
        let repository = MockAuditRepository::new();

        // Create old event
        let mut old_event = AuditEvent::new(AuditAction::Login, AuditSeverity::Low);
        old_event.timestamp = Utc::now() - chrono::Duration::days(100);

        // Create new event
        let new_event = AuditEvent::new(AuditAction::Login, AuditSeverity::Low);

        repository.store_event(&old_event).await.unwrap();
        repository.store_event(&new_event).await.unwrap();

        let removed_count = repository.cleanup_old_events(90).await.unwrap();
        assert_eq!(removed_count, 1);

        let remaining_events = repository.get_all_events();
        assert_eq!(remaining_events.len(), 1);
    }
}
