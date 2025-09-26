// 핵심 서비스 모듈
// 인증, 데이터베이스, 설정, 로깅 등 시스템의 핵심 기능들

pub mod auth;
pub mod config;
pub mod database;
pub mod logging;
pub mod ops;
pub mod security;

pub use auth::{AuthService, LoginRequest};
pub use config::{AppConfig, AuthConfig, DatabaseConfig, LoggingConfig};
pub use database::{
    DatabaseConnection, DatabaseManager, PoolInfo, QueryLogger, TransactionManager,
};
pub use logging::DatabaseLogger;
