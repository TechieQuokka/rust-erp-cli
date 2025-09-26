use crate::core::auth::service::{AuthService, AuthenticatedUser};
use crate::core::security::rate_limiter::RateLimiterTrait;
use crate::utils::error::{ErpError, ErpResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user: Option<AuthenticatedUser>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_timestamp: DateTime<Utc>,
    pub permissions: Vec<String>,
}

impl SecurityContext {
    pub fn new() -> Self {
        Self {
            user: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            request_timestamp: Utc::now(),
            permissions: Vec::new(),
        }
    }

    pub fn with_user(mut self, user: AuthenticatedUser) -> Self {
        self.permissions = user.permissions.clone();
        self.user = Some(user);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_client_info(mut self, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    pub fn user_id(&self) -> Option<Uuid> {
        self.user.as_ref().map(|u| u.id)
    }

    pub fn username(&self) -> Option<&str> {
        self.user.as_ref().map(|u| u.username.as_str())
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub name: String,
    pub enabled: bool,
    pub required_permissions: Vec<String>,
    pub allowed_ip_ranges: Vec<String>,
    pub rate_limit: Option<RateLimit>,
    pub require_session: bool,
    pub max_session_age_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub session_timeout_hours: i64,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: i64,
    pub require_https: bool,
    pub secure_headers: bool,
    pub cors_enabled: bool,
    pub allowed_origins: Vec<String>,
    pub csrf_protection: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            session_timeout_hours: 8,
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            require_https: true,
            secure_headers: true,
            cors_enabled: false,
            allowed_origins: vec!["localhost".to_string()],
            csrf_protection: true,
        }
    }
}

pub struct SecurityMiddleware {
    auth_service: Arc<AuthService>,
    rate_limiter: Arc<dyn RateLimiterTrait>,
    config: SecurityConfig,
    rules: HashMap<String, SecurityRule>,
}

impl SecurityMiddleware {
    pub fn new(
        auth_service: Arc<AuthService>,
        rate_limiter: Arc<dyn RateLimiterTrait>,
        config: SecurityConfig,
    ) -> Self {
        Self {
            auth_service,
            rate_limiter,
            config,
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, endpoint: String, rule: SecurityRule) {
        self.rules.insert(endpoint, rule);
    }

    pub async fn process_request(
        &self,
        endpoint: &str,
        token: Option<&str>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
    ) -> ErpResult<SecurityContext> {
        let mut context = SecurityContext::new()
            .with_client_info(ip_address.clone(), user_agent);

        if let Some(sid) = session_id {
            context = context.with_session(sid);
        }

        // Apply rate limiting
        if let Some(client_ip) = &ip_address {
            if !self.rate_limiter.allow_request(client_ip).await? {
                warn!("Rate limit exceeded for IP: {}", client_ip);
                return Err(ErpError::validation("rate_limit", "too many requests"));
            }
        }

        // Check security rules for endpoint
        if let Some(rule) = self.rules.get(endpoint) {
            if !rule.enabled {
                return Ok(context);
            }

            // Check IP restrictions
            if !rule.allowed_ip_ranges.is_empty() {
                if let Some(ref ip) = ip_address {
                    if !self.is_ip_allowed(ip, &rule.allowed_ip_ranges) {
                        warn!("IP address {} not allowed for endpoint {}", ip, endpoint);
                        return Err(ErpError::forbidden("IP address not allowed"));
                    }
                }
            }

            // Apply endpoint-specific rate limiting
            if let Some(ref rate_limit) = rule.rate_limit {
                if let Some(ref ip) = ip_address {
                    let key = format!("{}:{}", endpoint, ip);
                    if !self.rate_limiter.check_rate_limit(&key, rate_limit.requests_per_minute as u64, 60).await? {
                        return Err(ErpError::validation("rate_limit", "endpoint rate limit exceeded"));
                    }
                }
            }
        }

        // Authenticate user if token is provided
        if let Some(token_str) = token {
            match self.auth_service.get_authenticated_user(token_str).await {
                Ok(user) => {
                    context = context.with_user(user);
                    info!("User authenticated: {}", context.username().unwrap_or("unknown"));
                }
                Err(e) => {
                    warn!("Authentication failed: {}", e);
                    return Err(ErpError::Authentication("Invalid or expired token".to_string()));
                }
            }
        }

        // Check endpoint-specific requirements
        if let Some(rule) = self.rules.get(endpoint) {
            // Check if authentication is required
            if rule.require_session && !context.is_authenticated() {
                return Err(ErpError::Authentication("Authentication required".to_string()));
            }

            // Check permissions
            for permission in &rule.required_permissions {
                if !context.has_permission(permission) {
                    warn!("User {} lacks permission: {}",
                          context.username().unwrap_or("anonymous"), permission);
                    return Err(ErpError::forbidden(&format!("Missing permission: {}", permission)));
                }
            }

            // Check session age
            if let Some(max_age) = rule.max_session_age_hours {
                if let Some(user) = &context.user {
                    if let Some(last_login) = user.last_login_at {
                        let age = Utc::now().signed_duration_since(last_login);
                        if age.num_hours() > max_age {
                            warn!("Session expired for user: {}", user.username);
                            return Err(ErpError::Authentication("Session expired".to_string()));
                        }
                    }
                }
            }
        }

        Ok(context)
    }

    pub async fn validate_permission(&self, context: &SecurityContext, permission: &str) -> ErpResult<()> {
        if !context.has_permission(permission) {
            return Err(ErpError::forbidden(&format!("Missing permission: {}", permission)));
        }
        Ok(())
    }

    pub async fn validate_resource_access(
        &self,
        context: &SecurityContext,
        resource: &str,
        _resource_id: &str,
        action: &str,
    ) -> ErpResult<()> {
        let permission = format!("{}:{}", resource, action);
        self.validate_permission(context, &permission).await?;

        // Additional resource-specific checks can be added here
        // For example, checking if user owns the resource

        Ok(())
    }

    pub fn get_security_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if self.config.secure_headers {
            headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
            headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
            headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
            headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());

            if self.config.require_https {
                headers.insert("Strict-Transport-Security".to_string(),
                             "max-age=31536000; includeSubDomains".to_string());
            }
        }

        if self.config.cors_enabled {
            let allowed_origins = self.config.allowed_origins.join(", ");
            headers.insert("Access-Control-Allow-Origin".to_string(), allowed_origins);
            headers.insert("Access-Control-Allow-Methods".to_string(),
                         "GET, POST, PUT, DELETE, OPTIONS".to_string());
            headers.insert("Access-Control-Allow-Headers".to_string(),
                         "Authorization, Content-Type, X-Requested-With".to_string());
        }

        headers
    }

    fn is_ip_allowed(&self, ip: &str, allowed_ranges: &[String]) -> bool {
        // Simple IP checking - in production, use a proper IP range library
        for range in allowed_ranges {
            if range == "*" || range == ip {
                return true;
            }

            // Simple CIDR-like checking (very basic)
            if range.ends_with("/24") {
                let network = range.replace("/24", "");
                if ip.starts_with(&network) {
                    return true;
                }
            }
        }
        false
    }

    pub async fn cleanup_expired_sessions(&self) -> ErpResult<usize> {
        // This would typically clean up expired sessions from a session store
        // For now, delegate to the auth service
        self.auth_service.cleanup_expired_sessions().await
    }

    pub async fn revoke_all_user_sessions(&self, user_id: Uuid) -> ErpResult<()> {
        self.auth_service.terminate_all_sessions(user_id).await
    }
}

// Security middleware builder pattern
pub struct SecurityMiddlewareBuilder {
    auth_service: Option<Arc<AuthService>>,
    rate_limiter: Option<Arc<dyn RateLimiterTrait>>,
    config: SecurityConfig,
    rules: HashMap<String, SecurityRule>,
}

impl SecurityMiddlewareBuilder {
    pub fn new() -> Self {
        Self {
            auth_service: None,
            rate_limiter: None,
            config: SecurityConfig::default(),
            rules: HashMap::new(),
        }
    }

    pub fn with_auth_service(mut self, auth_service: Arc<AuthService>) -> Self {
        self.auth_service = Some(auth_service);
        self
    }

    pub fn with_rate_limiter(mut self, rate_limiter: Arc<dyn RateLimiterTrait>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    pub fn with_config(mut self, config: SecurityConfig) -> Self {
        self.config = config;
        self
    }

    pub fn add_rule(mut self, endpoint: String, rule: SecurityRule) -> Self {
        self.rules.insert(endpoint, rule);
        self
    }

    pub fn build(self) -> ErpResult<SecurityMiddleware> {
        let auth_service = self.auth_service
            .ok_or_else(|| ErpError::internal("Auth service is required"))?;
        let rate_limiter = self.rate_limiter
            .ok_or_else(|| ErpError::internal("Rate limiter is required"))?;

        let mut middleware = SecurityMiddleware::new(auth_service, rate_limiter, self.config);

        for (endpoint, rule) in self.rules {
            middleware.add_rule(endpoint, rule);
        }

        Ok(middleware)
    }
}

impl Default for SecurityMiddlewareBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::jwt::JwtService;
    use crate::core::auth::rbac::RbacService;
    use crate::core::auth::service::{AuthConfig, MockUserRepository};
    use crate::core::security::rate_limiter::MockRateLimiter;
    use crate::utils::crypto::HashingService;
    use crate::utils::validation::ValidationService;

    fn create_test_middleware() -> SecurityMiddleware {
        let user_repo = Arc::new(MockUserRepository::new());
        let jwt_service = JwtService::new(crate::core::auth::jwt::JwtConfig::default());
        let rbac_service = RbacService::new();
        let hashing_service = HashingService::new();
        let validation_service = ValidationService::new();
        let auth_config = AuthConfig::default();

        let auth_service = Arc::new(AuthService::new(
            user_repo,
            jwt_service,
            rbac_service,
            hashing_service,
            validation_service,
            auth_config,
        ));

        let rate_limiter: Arc<dyn RateLimiterTrait> = Arc::new(MockRateLimiter::new());
        let config = SecurityConfig::default();

        SecurityMiddleware::new(auth_service, rate_limiter, config)
    }

    #[tokio::test]
    async fn test_security_context_creation() {
        let context = SecurityContext::new()
            .with_client_info(Some("127.0.0.1".to_string()), Some("test-agent".to_string()))
            .with_session("session123".to_string());

        assert!(!context.is_authenticated());
        assert_eq!(context.ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(context.user_agent, Some("test-agent".to_string()));
        assert_eq!(context.session_id, Some("session123".to_string()));
    }

    #[tokio::test]
    async fn test_security_rule_creation() {
        let rule = SecurityRule {
            name: "admin_only".to_string(),
            enabled: true,
            required_permissions: vec!["admin".to_string()],
            allowed_ip_ranges: vec!["192.168.1.0/24".to_string()],
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                burst_size: 10,
            }),
            require_session: true,
            max_session_age_hours: Some(8),
        };

        assert!(rule.enabled);
        assert!(rule.require_session);
        assert_eq!(rule.required_permissions.len(), 1);
    }

    #[tokio::test]
    async fn test_middleware_without_authentication() {
        let middleware = create_test_middleware();

        let result = middleware.process_request(
            "/public",
            None,
            Some("127.0.0.1".to_string()),
            None,
            None,
        ).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.is_authenticated());
    }

    #[tokio::test]
    async fn test_ip_allowlist() {
        let middleware = create_test_middleware();

        // Test basic IP matching
        assert!(middleware.is_ip_allowed("192.168.1.1", &["192.168.1.1".to_string()]));
        assert!(middleware.is_ip_allowed("10.0.0.1", &["*".to_string()]));
        assert!(!middleware.is_ip_allowed("192.168.1.1", &["10.0.0.1".to_string()]));

        // Test CIDR-like matching (basic)
        assert!(middleware.is_ip_allowed("192.168.1.1", &["192.168.1/24".to_string()]));
        assert!(!middleware.is_ip_allowed("10.0.0.1", &["192.168.1/24".to_string()]));
    }

    #[tokio::test]
    async fn test_security_headers() {
        let config = SecurityConfig {
            secure_headers: true,
            require_https: true,
            cors_enabled: true,
            allowed_origins: vec!["https://example.com".to_string()],
            ..Default::default()
        };

        let test_middleware = create_test_middleware();
        let auth_service = test_middleware.auth_service.clone();
        let rate_limiter: Arc<dyn RateLimiterTrait> = Arc::new(MockRateLimiter::new());
        let middleware = SecurityMiddleware::new(auth_service, rate_limiter, config);

        let headers = middleware.get_security_headers();

        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("Strict-Transport-Security"));
        assert!(headers.contains_key("Access-Control-Allow-Origin"));
        assert_eq!(headers.get("X-Content-Type-Options"), Some(&"nosniff".to_string()));
    }

    #[tokio::test]
    async fn test_middleware_builder() {
        let user_repo = Arc::new(MockUserRepository::new());
        let jwt_service = JwtService::new(crate::core::auth::jwt::JwtConfig::default());
        let rbac_service = RbacService::new();
        let hashing_service = HashingService::new();
        let validation_service = ValidationService::new();
        let auth_config = AuthConfig::default();

        let auth_service = Arc::new(AuthService::new(
            user_repo,
            jwt_service,
            rbac_service,
            hashing_service,
            validation_service,
            auth_config,
        ));

        let rate_limiter: Arc<dyn RateLimiterTrait> = Arc::new(MockRateLimiter::new());

        let rule = SecurityRule {
            name: "test_rule".to_string(),
            enabled: true,
            required_permissions: vec!["read".to_string()],
            allowed_ip_ranges: vec![],
            rate_limit: None,
            require_session: false,
            max_session_age_hours: None,
        };

        let middleware = SecurityMiddlewareBuilder::new()
            .with_auth_service(auth_service)
            .with_rate_limiter(rate_limiter)
            .add_rule("/test".to_string(), rule)
            .build();

        assert!(middleware.is_ok());
    }
}