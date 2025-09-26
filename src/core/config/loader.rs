use crate::core::config::AppConfig;
use crate::utils::error::{ErpError, ErpResult};
use config::{Config, Environment, File};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

static CONFIG_INSTANCE: OnceLock<RwLock<Option<AppConfig>>> = OnceLock::new();

pub struct ConfigLoader {
    config_paths: Vec<PathBuf>,
    environment: String,
}

impl ConfigLoader {
    pub fn new() -> Self {
        let environment = env::var("ERP_ENV").unwrap_or_else(|_| "development".to_string());

        Self {
            config_paths: Self::get_config_paths(),
            environment,
        }
    }

    pub fn with_environment<S: Into<String>>(environment: S) -> Self {
        Self {
            config_paths: Self::get_config_paths(),
            environment: environment.into(),
        }
    }

    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        let environment = env::var("ERP_ENV").unwrap_or_else(|_| "development".to_string());

        Self {
            config_paths: paths,
            environment,
        }
    }

    pub async fn load(&self) -> ErpResult<AppConfig> {
        info!(
            "Loading configuration for environment: {}",
            self.environment
        );

        let mut config_builder = Config::builder();

        for config_path in &self.config_paths {
            config_builder = self.add_config_files(config_builder, config_path)?;
        }

        let mut config = config_builder
            .add_source(Environment::with_prefix("ERP").separator("_"))
            .build()
            .map_err(|e| ErpError::config(format!("Failed to build configuration: {}", e)))?;

        self.apply_environment_overrides(&mut config)?;

        let app_config: AppConfig = config
            .try_deserialize()
            .map_err(|e| ErpError::config(format!("Failed to deserialize configuration: {}", e)))?;

        self.validate_config(&app_config)?;

        debug!("Configuration loaded successfully");
        Ok(app_config)
    }

    pub async fn load_and_cache(&self) -> ErpResult<()> {
        let config = self.load().await?;
        let config_lock = CONFIG_INSTANCE.get_or_init(|| RwLock::new(None));
        let mut guard = config_lock.write().await;
        *guard = Some(config);

        info!("Configuration cached successfully");
        Ok(())
    }

    pub async fn reload(&self) -> ErpResult<()> {
        info!("Reloading configuration");
        self.load_and_cache().await
    }

    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Ok(config_dir) = env::var("ERP_CONFIG_DIR") {
            paths.push(PathBuf::from(config_dir));
        }

        paths.push(PathBuf::from("config"));
        paths.push(PathBuf::from("./config"));
        paths.push(PathBuf::from("../config"));

        if let Ok(home) = env::var("HOME") {
            paths.push(PathBuf::from(home).join(".erp"));
        }

        if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
            paths.push(PathBuf::from(config_home).join("erp"));
        }

        paths
    }

    fn add_config_files(
        &self,
        mut builder: config::ConfigBuilder<config::builder::DefaultState>,
        config_path: &Path,
    ) -> ErpResult<config::ConfigBuilder<config::builder::DefaultState>> {
        if !config_path.exists() {
            debug!("Config path does not exist: {}", config_path.display());
            return Ok(builder);
        }

        let default_file = config_path.join("default");
        if self.file_exists_with_extensions(&default_file) {
            debug!("Loading default config from: {}", default_file.display());
            builder = builder.add_source(File::from(default_file).required(false));
        }

        let env_file = config_path.join(&self.environment);
        if self.file_exists_with_extensions(&env_file) {
            info!("Loading environment config from: {}", env_file.display());
            builder = builder.add_source(File::from(env_file).required(false));
        }

        let local_file = config_path.join("local");
        if self.file_exists_with_extensions(&local_file) {
            debug!("Loading local config from: {}", local_file.display());
            builder = builder.add_source(File::from(local_file).required(false));
        }

        Ok(builder)
    }

    fn file_exists_with_extensions(&self, base_path: &Path) -> bool {
        let extensions = ["toml", "yaml", "yml", "json"];

        for ext in extensions {
            let file_path = base_path.with_extension(ext);
            if file_path.exists() {
                return true;
            }
        }

        false
    }

    fn apply_environment_overrides(&self, config: &mut Config) -> ErpResult<()> {
        let env_mappings = [
            ("DATABASE_URL", "database.url"),
            ("JWT_SECRET", "auth.jwt_secret"),
            ("LOG_LEVEL", "logging.level"),
            ("DB_MAX_CONNECTIONS", "database.max_connections"),
            ("TOKEN_EXPIRY_HOURS", "auth.token_expiry_hours"),
            ("PASSWORD_MIN_LENGTH", "auth.password_min_length"),
        ];

        for (env_var, config_key) in env_mappings {
            if let Ok(value) = env::var(env_var) {
                debug!(
                    "Applying environment override: {} -> {}",
                    env_var, config_key
                );
                config.set(config_key, value).map_err(|e| {
                    ErpError::config(format!(
                        "Failed to set config from env var {}: {}",
                        env_var, e
                    ))
                })?;
            }
        }

        Ok(())
    }

    fn validate_config(&self, config: &AppConfig) -> ErpResult<()> {
        if config.database.url.is_empty() {
            return Err(ErpError::config("Database URL cannot be empty"));
        }

        if config.auth.jwt_secret.is_empty() {
            return Err(ErpError::config("JWT secret cannot be empty"));
        }

        if config.auth.jwt_secret.len() < 32 {
            warn!("JWT secret is less than 32 characters, consider using a longer secret");
        }

        if config.database.max_connections == 0 {
            return Err(ErpError::config(
                "Database max connections must be greater than 0",
            ));
        }

        if config.auth.token_expiry_hours == 0 {
            return Err(ErpError::config(
                "Token expiry hours must be greater than 0",
            ));
        }

        if config.auth.password_min_length < 8 {
            warn!("Password minimum length is less than 8, consider increasing for security");
        }

        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn get_config() -> ErpResult<AppConfig> {
    let config_lock = CONFIG_INSTANCE.get_or_init(|| RwLock::new(None));
    let guard = config_lock.read().await;

    match guard.as_ref() {
        Some(config) => Ok(config.clone()),
        None => {
            drop(guard);
            let loader = ConfigLoader::new();
            loader.load_and_cache().await?;

            let guard = config_lock.read().await;
            guard
                .as_ref()
                .cloned()
                .ok_or_else(|| ErpError::internal("Failed to load configuration"))
        }
    }
}

pub async fn reload_config() -> ErpResult<()> {
    let loader = ConfigLoader::new();
    loader.reload().await
}

pub async fn is_config_loaded() -> bool {
    let config_lock = CONFIG_INSTANCE.get_or_init(|| RwLock::new(None));
    let guard = config_lock.read().await;
    guard.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.environment, "development");
    }

    #[tokio::test]
    async fn test_config_loader_with_environment() {
        let loader = ConfigLoader::with_environment("production");
        assert_eq!(loader.environment, "production");
    }

    #[tokio::test]
    async fn test_file_exists_with_extensions() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("test.toml");
        fs::write(&config_file, "key = \"value\"").unwrap();

        let loader = ConfigLoader::new();
        let base_path = temp_dir.path().join("test");

        assert!(loader.file_exists_with_extensions(&base_path));
    }

    #[tokio::test]
    async fn test_validate_config_empty_database_url() {
        let mut config = create_test_config();
        config.database.url = String::new();

        let loader = ConfigLoader::new();
        let result = loader.validate_config(&config);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Database URL cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_config_empty_jwt_secret() {
        let mut config = create_test_config();
        config.auth.jwt_secret = String::new();

        let loader = ConfigLoader::new();
        let result = loader.validate_config(&config);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("JWT secret cannot be empty"));
    }

    fn create_test_config() -> AppConfig {
        use crate::core::config::{AuthConfig, DatabaseConfig, LoggingConfig};

        AppConfig {
            database: DatabaseConfig {
                url: "sqlite::memory:".to_string(),
                max_connections: 10,
                migrate_on_start: true,
                query_timeout_seconds: 30,
                idle_timeout_seconds: 600,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file: "logs/erp.log".to_string(),
                rotate_daily: true,
                max_file_size: "10MB".to_string(),
                max_files: 7,
            },
            auth: AuthConfig {
                jwt_secret: "test_secret_key_that_is_long_enough".to_string(),
                token_expiry_hours: 24,
                password_min_length: 8,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
            },
        }
    }
}
