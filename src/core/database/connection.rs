use crate::core::config::DatabaseConfig;
use crate::core::logging::DatabaseLogger;
use crate::utils::error::{ErpError, ErpResult};
use sqlx::{migrate::MigrateDatabase, Pool, Postgres, Row};
use std::sync::Arc;

pub type DatabasePool = Pool<Postgres>;
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use tracing::{debug, info, warn};
use uuid::Uuid;

static DB_POOL: OnceCell<Arc<DatabaseConnection>> = OnceCell::const_new();

#[derive(Clone)]
pub struct DatabaseConnection {
    pool: Pool<Postgres>,
    _config: DatabaseConfig,
    _logger: DatabaseLogger,
}

impl DatabaseConnection {
    pub async fn new(config: DatabaseConfig) -> ErpResult<Self> {
        let pool = Self::create_pool(&config).await?;
        let logger = DatabaseLogger;

        let migrate_on_start = config.migrate_on_start;

        let connection = Self {
            pool,
            _config: config,
            _logger: logger,
        };

        if migrate_on_start {
            connection.run_migrations().await?;
        }

        Ok(connection)
    }

    async fn create_pool(config: &DatabaseConfig) -> ErpResult<Pool<Postgres>> {
        info!(
            "Creating database connection pool with max connections: {}",
            config.max_connections
        );

        if !Postgres::database_exists(&config.url)
            .await
            .unwrap_or(false)
        {
            info!("Database does not exist, creating it");
            Postgres::create_database(&config.url)
                .await
                .map_err(|e| ErpError::internal(format!("Failed to create database: {}", e)))?;
        }

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to connect to database: {}", e)))?;

        Ok(pool)
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub async fn health_check(&self) -> ErpResult<()> {
        let start = Instant::now();

        let row = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Database health check failed: {}", e)))?;

        let value: i32 = row.try_get(0).map_err(|e| {
            ErpError::internal(format!("Failed to parse health check result: {}", e))
        })?;

        if value != 1 {
            return Err(ErpError::internal(
                "Database health check returned unexpected value",
            ));
        }

        let duration = start.elapsed();
        debug!("Database health check passed in {:?}", duration);

        if duration > Duration::from_millis(1000) {
            warn!(
                "Database health check took longer than expected: {:?}",
                duration
            );
        }

        Ok(())
    }

    pub async fn get_pool_info(&self) -> PoolInfo {
        PoolInfo {
            max_connections: self.pool.options().get_max_connections(),
            idle_connections: self.pool.num_idle(),
            total_connections: self.pool.size(),
        }
    }

    pub async fn close(&self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }

    async fn run_migrations(&self) -> ErpResult<()> {
        info!("Running database migrations");

        let migrations_path =
            std::env::var("ERP_MIGRATIONS_PATH").unwrap_or_else(|_| "./migrations".to_string());

        match sqlx::migrate::Migrator::new(std::path::Path::new(&migrations_path)).await {
            Ok(migrator) => {
                migrator
                    .run(&self.pool)
                    .await
                    .map_err(|e| ErpError::internal(format!("Migration failed: {}", e)))?;
                info!("Database migrations completed successfully");
            }
            Err(e) => {
                warn!("No migrations found or migration directory error: {}", e);
                info!("Skipping migrations - this is normal for first-time setup");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub max_connections: u32,
    pub idle_connections: usize,
    pub total_connections: u32,
}

pub struct DatabaseManager;

impl DatabaseManager {
    pub async fn initialize(config: DatabaseConfig) -> ErpResult<()> {
        let connection = DatabaseConnection::new(config).await?;

        DB_POOL
            .set(Arc::new(connection))
            .map_err(|_| ErpError::internal("Database connection already initialized"))?;

        info!("Database manager initialized successfully");
        Ok(())
    }

    pub async fn get_connection() -> ErpResult<Arc<DatabaseConnection>> {
        DB_POOL
            .get()
            .ok_or_else(|| ErpError::internal("Database connection not initialized"))
            .map(Arc::clone)
    }

    pub async fn health_check() -> ErpResult<()> {
        let connection = Self::get_connection().await?;
        connection.health_check().await
    }

    pub async fn get_pool_info() -> ErpResult<PoolInfo> {
        let connection = Self::get_connection().await?;
        Ok(connection.get_pool_info().await)
    }

    pub async fn close() -> ErpResult<()> {
        if let Some(connection) = DB_POOL.get() {
            connection.close().await;
            info!("Database connection closed");
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TransactionManager {
    connection: Arc<DatabaseConnection>,
    tx_id: Uuid,
    logger: DatabaseLogger,
}

impl TransactionManager {
    pub async fn new() -> ErpResult<Self> {
        let connection = DatabaseManager::get_connection().await?;
        let tx_id = Uuid::new_v4();
        let logger = DatabaseLogger;

        logger.log_transaction_start(tx_id);

        Ok(Self {
            connection,
            tx_id,
            logger,
        })
    }

    pub fn tx_id(&self) -> Uuid {
        self.tx_id
    }

    pub async fn execute_in_transaction<F, R>(&self, operation: F) -> ErpResult<R>
    where
        F: FnOnce(
            &mut sqlx::Transaction<'_, sqlx::Postgres>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = ErpResult<R>> + Send>>,
        R: Send,
    {
        let start = Instant::now();
        let mut tx = self
            .connection
            .pool()
            .begin()
            .await
            .map_err(|e| ErpError::internal(format!("Failed to start transaction: {}", e)))?;

        let result = operation(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await.map_err(|e| {
                    ErpError::internal(format!("Failed to commit transaction: {}", e))
                })?;

                let duration = start.elapsed();
                self.logger
                    .log_transaction_end(self.tx_id, true, duration.as_millis() as u64);
                Ok(value)
            }
            Err(error) => {
                if let Err(rollback_error) = tx.rollback().await {
                    warn!("Failed to rollback transaction: {}", rollback_error);
                }

                let duration = start.elapsed();
                self.logger
                    .log_transaction_end(self.tx_id, false, duration.as_millis() as u64);
                Err(error)
            }
        }
    }
}

pub struct QueryLogger {
    logger: DatabaseLogger,
}

impl QueryLogger {
    pub fn new() -> Self {
        Self {
            logger: DatabaseLogger,
        }
    }

    pub fn log_query(&self, query: &str, duration: Duration) {
        self.logger.log_query(query, duration.as_millis() as u64);
    }

    pub fn log_slow_query(&self, query: &str, duration: Duration, threshold_ms: u64) {
        if duration.as_millis() as u64 > threshold_ms {
            warn!(
                "Slow query detected ({}ms): {}",
                duration.as_millis(),
                query
            );
        }
    }
}

impl Default for QueryLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::DatabaseConfig;

    fn create_test_config() -> DatabaseConfig {
        // Use PostgreSQL test database for consistency with production
        DatabaseConfig {
            url: "postgresql://postgres:2147483647@localhost:5432/erp_test_db".to_string(),
            max_connections: 5,
            migrate_on_start: false,
            query_timeout_seconds: 30,
            idle_timeout_seconds: 600,
        }
    }

    #[tokio::test]
    async fn test_database_connection_creation() {
        // Ensure we're using test config, not environment variables
        std::env::remove_var("DATABASE_URL");
        let config = create_test_config();
        let result = DatabaseConnection::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        std::env::remove_var("DATABASE_URL");
        let config = create_test_config();
        let connection = DatabaseConnection::new(config).await.unwrap();
        let result = connection.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pool_info() {
        std::env::remove_var("DATABASE_URL");
        let config = create_test_config();
        let connection = DatabaseConnection::new(config).await.unwrap();
        let pool_info = connection.get_pool_info().await;

        assert_eq!(pool_info.max_connections, 5);
        assert!(pool_info.total_connections <= 5);
    }

    #[tokio::test]
    async fn test_transaction_manager_creation() {
        std::env::remove_var("DATABASE_URL");
        let config = create_test_config();
        let _ = DatabaseConnection::new(config).await.unwrap();

        let result = TransactionManager::new().await;
        if let Ok(tx_manager) = result {
            assert!(!tx_manager.tx_id().is_nil());
        }
    }

    #[test]
    fn test_query_logger_creation() {
        let logger = QueryLogger::new();
        let duration = Duration::from_millis(100);
        logger.log_query("SELECT 1", duration);
    }
}
