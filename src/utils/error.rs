use thiserror::Error;
use uuid::Uuid;

/// ERP 시스템의 커스텀 에러 타입
#[derive(Debug, Error)]
pub enum ErpError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Authentication failed: {reason}")]
    Auth { reason: String },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Authorization failed: user {user_id} cannot {action} on {resource}")]
    Forbidden {
        user_id: Uuid,
        action: String,
        resource: String,
    },

    #[error("Validation error: {field} is {reason}")]
    Validation { field: String, reason: String },

    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Business rule violation: {rule}")]
    BusinessRule { rule: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("External service error: {service} returned {status}")]
    ExternalService { service: String, status: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    ConfigLoad(#[from] config::ConfigError),

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// ERP 시스템의 결과 타입
pub type ErpResult<T> = Result<T, ErpError>;

impl ErpError {
    /// 설정 에러 생성
    pub fn config<T: Into<String>>(message: T) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 인증 에러 생성
    pub fn auth<T: Into<String>>(reason: T) -> Self {
        Self::Auth {
            reason: reason.into(),
        }
    }

    /// 검증 에러 생성
    pub fn validation<T: Into<String>, U: Into<String>>(field: T, reason: U) -> Self {
        Self::Validation {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// 리소스 찾을 수 없음 에러 생성
    pub fn not_found<T: Into<String>, U: Into<String>>(resource_type: T, id: U) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// 비즈니스 규칙 위반 에러 생성
    pub fn business_rule<T: Into<String>>(rule: T) -> Self {
        Self::BusinessRule { rule: rule.into() }
    }

    /// 내부 에러 생성
    pub fn internal<T: Into<String>>(message: T) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// 간단한 검증 에러 생성 (필드명 없이)
    pub fn validation_simple<T: Into<String>>(reason: T) -> Self {
        Self::Validation {
            field: "input".into(),
            reason: reason.into(),
        }
    }

    /// 간단한 NotFound 에러 생성
    pub fn not_found_simple<T: Into<String>>(message: T) -> Self {
        Self::NotFound {
            resource_type: "resource".into(),
            id: message.into(),
        }
    }

    /// Conflict 에러 생성
    pub fn conflict<T: Into<String>>(message: T) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// IO 에러 생성 (helper method)
    pub fn io<T: Into<String>>(message: T) -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            message.into(),
        ))
    }

    /// 직렬화 에러 생성
    pub fn serialization<T: Into<String>>(message: T) -> Self {
        Self::Internal {
            message: format!("Serialization error: {}", message.into()),
        }
    }

    /// 지원되지 않는 기능 에러 생성
    pub fn unsupported<T: Into<String>>(message: T) -> Self {
        Self::Internal {
            message: format!("Unsupported: {}", message.into()),
        }
    }

    /// 구현되지 않은 기능 에러 생성
    pub fn not_implemented<T: Into<String>>(message: T) -> Self {
        Self::Internal {
            message: format!("Not implemented: {}", message.into()),
        }
    }

    /// 데이터베이스 에러 생성 (helper method)
    pub fn database<T: Into<String>>(message: T) -> Self {
        Self::Internal {
            message: format!("Database error: {}", message.into()),
        }
    }

    /// 권한 거부 에러 생성 (간단한 버전)
    pub fn forbidden<T: Into<String>>(message: T) -> Self {
        Self::Authorization(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_error_creation_and_display() {
        let config_err = ErpError::config("Invalid config format");
        assert!(config_err.to_string().contains("Configuration error"));
        assert!(config_err.to_string().contains("Invalid config format"));

        let auth_err = ErpError::auth("Invalid credentials");
        assert!(auth_err.to_string().contains("Authentication failed"));
        assert!(auth_err.to_string().contains("Invalid credentials"));

        let validation_err = ErpError::validation("email", "invalid format");
        assert!(validation_err.to_string().contains("Validation error"));
        assert!(validation_err
            .to_string()
            .contains("email is invalid format"));

        let not_found_err = ErpError::not_found("User", "123");
        assert!(not_found_err.to_string().contains("Resource not found"));
        assert!(not_found_err.to_string().contains("User with id 123"));

        let business_rule_err = ErpError::business_rule("Stock cannot be negative");
        assert!(business_rule_err
            .to_string()
            .contains("Business rule violation"));
        assert!(business_rule_err
            .to_string()
            .contains("Stock cannot be negative"));

        let internal_err = ErpError::internal("System error");
        assert!(internal_err.to_string().contains("Internal error"));
        assert!(internal_err.to_string().contains("System error"));
    }

    #[test]
    fn test_simple_error_helpers() {
        let validation_simple = ErpError::validation_simple("invalid input");
        assert!(validation_simple
            .to_string()
            .contains("input is invalid input"));

        let not_found_simple = ErpError::not_found_simple("item not found");
        assert!(not_found_simple
            .to_string()
            .contains("resource with id item not found"));

        let conflict_err = ErpError::conflict("Duplicate entry");
        assert!(conflict_err
            .to_string()
            .contains("Conflict: Duplicate entry"));
    }

    #[test]
    fn test_complex_error_helpers() {
        let user_id = Uuid::new_v4();
        let forbidden_err = ErpError::Forbidden {
            user_id,
            action: "delete".to_string(),
            resource: "admin_user".to_string(),
        };
        let error_msg = forbidden_err.to_string();
        assert!(error_msg.contains("Authorization failed"));
        assert!(error_msg.contains("cannot delete"));
        assert!(error_msg.contains("admin_user"));

        let external_service_err = ErpError::ExternalService {
            service: "payment_gateway".to_string(),
            status: "500".to_string(),
        };
        assert!(external_service_err
            .to_string()
            .contains("External service error"));
        assert!(external_service_err
            .to_string()
            .contains("payment_gateway returned 500"));
    }

    #[test]
    fn test_error_conversion_from_std_errors() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let erp_error = ErpError::from(io_error);
        assert!(erp_error.to_string().contains("IO error"));
        assert!(erp_error.to_string().contains("File not found"));

        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());
        let erp_error = ErpError::from(json_error.unwrap_err());
        assert!(erp_error.to_string().contains("JSON error"));
    }

    #[test]
    fn test_specialized_error_helpers() {
        let io_err = ErpError::io("Custom IO error");
        assert!(io_err.to_string().contains("IO error"));
        assert!(io_err.to_string().contains("Custom IO error"));

        let serialization_err = ErpError::serialization("Failed to serialize data");
        assert!(serialization_err.to_string().contains("Internal error"));
        assert!(serialization_err
            .to_string()
            .contains("Serialization error"));

        let unsupported_err = ErpError::unsupported("Feature not implemented");
        assert!(unsupported_err.to_string().contains("Internal error"));
        assert!(unsupported_err.to_string().contains("Unsupported"));

        let database_err = ErpError::database("Connection timeout");
        assert!(database_err.to_string().contains("Internal error"));
        assert!(database_err.to_string().contains("Database error"));

        let forbidden_simple = ErpError::forbidden("Access denied");
        assert!(forbidden_simple
            .to_string()
            .contains("Authorization failed"));
        assert!(forbidden_simple.to_string().contains("Access denied"));
    }
}
