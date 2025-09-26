// 설정 관리 모듈

pub mod loader;

use crate::utils::error::ErpResult;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

pub use loader::{get_config, is_config_loaded, reload_config, ConfigLoader};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub migrate_on_start: bool,
    pub query_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 10,
            migrate_on_start: false,
            query_timeout_seconds: 30,
            idle_timeout_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: String,
    pub rotate_daily: bool,
    pub max_file_size: String,
    pub max_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "full".to_string(),
            file: "logs/erp.log".to_string(),
            rotate_daily: true,
            max_file_size: "10MB".to_string(),
            max_files: 7,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: u64,
    pub password_min_length: u32,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default_jwt_secret_change_in_production".to_string(),
            token_expiry_hours: 24,
            password_min_length: 8,
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub auth: AuthConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            auth: AuthConfig::default(),
        }
    }
}

impl AppConfig {
    /// 설정 로드
    pub async fn load() -> ErpResult<Self> {
        let environment = env::var("ERP_ENV").unwrap_or_else(|_| "development".to_string());

        let mut builder = Config::builder()
            // 기본 설정 로드
            .add_source(File::with_name("config/default").required(false))
            // 환경별 설정 로드
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // 환경변수로 오버라이드
            .add_source(Environment::with_prefix("ERP"));

        // 환경변수에서 DATABASE_URL, JWT_SECRET 등을 직접 처리
        if let Ok(database_url) = env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", database_url)?;
        }
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            builder = builder.set_override("auth.jwt_secret", jwt_secret)?;
        }

        let config = builder.build()?;
        let app_config: AppConfig = config.try_deserialize()?;
        Ok(app_config)
    }
}
