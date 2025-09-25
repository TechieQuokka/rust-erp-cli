use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErpError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Authentication failed: {reason}")]
    Auth { reason: String },

    #[error("Authorization failed: user {user_id} cannot {action} on {resource}")]
    Forbidden {
        user_id: uuid::Uuid,
        action: String,
        resource: String,
    },

    #[error("Validation error: {field} is {reason}")]
    Validation { field: String, reason: String },

    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Business rule violation: {rule}")]
    BusinessRule { rule: String },

    #[error("External service error: {service} returned {status}")]
    ExternalService { service: String, status: String },
}

pub type ErpResult<T> = Result<T, ErpError>;
