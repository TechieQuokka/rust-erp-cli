use crate::core::config::LoggingConfig;
use crate::utils::error::{ErpError, ErpResult};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

pub struct LoggerBuilder {
    config: LoggingConfig,
}

impl LoggerBuilder {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    pub fn build(self) -> ErpResult<()> {
        let env_filter = self.create_env_filter()?;

        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| ErpError::internal(format!("Failed to initialize logger: {}", e)))?;

        tracing::info!("Logging system initialized successfully");
        Ok(())
    }

    fn create_env_filter(&self) -> ErpResult<EnvFilter> {
        let level = match self.config.level.to_lowercase().as_str() {
            "trace" => "trace",
            "debug" => "debug",
            "info" => "info",
            "warn" => "warn",
            "error" => "error",
            _ => "info",
        };

        let filter = EnvFilter::new(format!("erp={},sqlx=warn,hyper=warn,tower=warn", level))
            .add_directive("tower_http=debug".parse().unwrap());

        Ok(filter)
    }
}

#[derive(Clone)]
pub struct DatabaseLogger;

impl DatabaseLogger {
    pub fn log_transaction_start(&self, tx_id: Uuid) {
        tracing::debug!("Transaction started: {}", tx_id);
    }

    pub fn log_transaction_end(&self, tx_id: Uuid, success: bool, duration_ms: u64) {
        if success {
            tracing::debug!(
                "Transaction completed successfully: {} ({}ms)",
                tx_id,
                duration_ms
            );
        } else {
            tracing::warn!("Transaction failed: {} ({}ms)", tx_id, duration_ms);
        }
    }

    pub fn log_query(&self, query: &str, duration_ms: u64) {
        tracing::debug!("Query executed: {} ({}ms)", query, duration_ms);
    }
}

pub struct Logger;

impl Logger {
    pub fn init(config: LoggingConfig) -> ErpResult<()> {
        LoggerBuilder::new(config).build()
    }

    pub fn create_request_id() -> String {
        Uuid::new_v4().to_string()
    }
}
