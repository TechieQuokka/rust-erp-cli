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
