use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPool, sqlite::SqlitePool, FromRow, Row};
use std::collections::HashMap;
use tracing::{error, info, warn};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub checksum: String,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: i64,
}

#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub checksum: String,
}

pub enum DatabaseMigrator {
    Postgres(PostgresMigrator),
    Sqlite(SqliteMigrator),
}

impl DatabaseMigrator {
    pub async fn initialize(&self) -> Result<()> {
        match self {
            DatabaseMigrator::Postgres(migrator) => migrator.initialize().await,
            DatabaseMigrator::Sqlite(migrator) => migrator.initialize().await,
        }
    }

    pub async fn get_applied_migrations(&self) -> Result<Vec<Migration>> {
        match self {
            DatabaseMigrator::Postgres(migrator) => migrator.get_applied_migrations().await,
            DatabaseMigrator::Sqlite(migrator) => migrator.get_applied_migrations().await,
        }
    }

    pub async fn apply_migration(&self, migration: &MigrationFile) -> Result<i64> {
        match self {
            DatabaseMigrator::Postgres(migrator) => migrator.apply_migration(migration).await,
            DatabaseMigrator::Sqlite(migrator) => migrator.apply_migration(migration).await,
        }
    }

    pub async fn rollback_migration(&self, migration: &MigrationFile) -> Result<()> {
        match self {
            DatabaseMigrator::Postgres(migrator) => migrator.rollback_migration(migration).await,
            DatabaseMigrator::Sqlite(migrator) => migrator.rollback_migration(migration).await,
        }
    }

    pub async fn migration_exists(&self, version: &str) -> Result<bool> {
        match self {
            DatabaseMigrator::Postgres(migrator) => migrator.migration_exists(version).await,
            DatabaseMigrator::Sqlite(migrator) => migrator.migration_exists(version).await,
        }
    }
}

pub struct PostgresMigrator {
    pool: PgPool,
}

pub struct SqliteMigrator {
    pool: SqlitePool,
}

pub struct MigrationRunner {
    migrator: DatabaseMigrator,
    migrations: Vec<MigrationFile>,
}

impl PostgresMigrator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SqliteMigrator {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl PostgresMigrator {
    pub async fn initialize(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                checksum VARCHAR(255) NOT NULL,
                executed_at TIMESTAMPTZ DEFAULT NOW(),
                execution_time_ms BIGINT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_schema_migrations_executed_at
            ON schema_migrations(executed_at);
        "#;

        sqlx::query(sql).execute(&self.pool).await?;
        info!("Initialized PostgreSQL migration schema");
        Ok(())
    }

    pub async fn get_applied_migrations(&self) -> Result<Vec<Migration>> {
        let migrations = sqlx::query_as::<_, Migration>(
            "SELECT version, name, checksum, executed_at, execution_time_ms
             FROM schema_migrations
             ORDER BY executed_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(migrations)
    }

    pub async fn apply_migration(&self, migration: &MigrationFile) -> Result<i64> {
        let start_time = std::time::Instant::now();

        let mut tx = self.pool.begin().await?;

        sqlx::query(&migration.up_sql).execute(&mut *tx).await?;

        let execution_time = start_time.elapsed().as_millis() as i64;

        sqlx::query(
            "INSERT INTO schema_migrations (version, name, checksum, execution_time_ms)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&migration.version)
        .bind(&migration.name)
        .bind(&migration.checksum)
        .bind(execution_time)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "Applied migration {} ({}) in {}ms",
            migration.version, migration.name, execution_time
        );

        Ok(execution_time)
    }

    pub async fn rollback_migration(&self, migration: &MigrationFile) -> Result<()> {
        if let Some(down_sql) = &migration.down_sql {
            let mut tx = self.pool.begin().await?;

            sqlx::query(down_sql).execute(&mut *tx).await?;

            sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
                .bind(&migration.version)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;

            info!(
                "Rolled back migration {} ({})",
                migration.version, migration.name
            );
        } else {
            warn!(
                "No rollback SQL provided for migration {} ({})",
                migration.version, migration.name
            );
            return Err(anyhow::anyhow!(
                "No rollback SQL provided for migration {}",
                migration.version
            ));
        }

        Ok(())
    }

    pub async fn migration_exists(&self, version: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = $1")
                .bind(version)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }
}

impl SqliteMigrator {
    pub async fn initialize(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                executed_at TEXT DEFAULT (datetime('now')),
                execution_time_ms INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_schema_migrations_executed_at
            ON schema_migrations(executed_at);
        "#;

        sqlx::query(sql).execute(&self.pool).await?;
        info!("Initialized SQLite migration schema");
        Ok(())
    }

    pub async fn get_applied_migrations(&self) -> Result<Vec<Migration>> {
        let rows = sqlx::query(
            "SELECT version, name, checksum, executed_at, execution_time_ms
             FROM schema_migrations
             ORDER BY executed_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut migrations = Vec::new();
        for row in rows {
            let executed_at_str: String = row.get("executed_at");
            let executed_at = DateTime::parse_from_rfc3339(&executed_at_str)
                .or_else(|_| {
                    DateTime::parse_from_str(&executed_at_str, "%Y-%m-%d %H:%M:%S").or_else(|_| {
                        DateTime::parse_from_str(&executed_at_str, "%Y-%m-%dT%H:%M:%S")
                    })
                })
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            migrations.push(Migration {
                version: row.get("version"),
                name: row.get("name"),
                checksum: row.get("checksum"),
                executed_at,
                execution_time_ms: row.get("execution_time_ms"),
            });
        }

        Ok(migrations)
    }

    pub async fn apply_migration(&self, migration: &MigrationFile) -> Result<i64> {
        let start_time = std::time::Instant::now();

        let mut tx = self.pool.begin().await?;

        sqlx::query(&migration.up_sql).execute(&mut *tx).await?;

        let execution_time = start_time.elapsed().as_millis() as i64;

        sqlx::query(
            "INSERT INTO schema_migrations (version, name, checksum, execution_time_ms)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&migration.version)
        .bind(&migration.name)
        .bind(&migration.checksum)
        .bind(execution_time)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "Applied migration {} ({}) in {}ms",
            migration.version, migration.name, execution_time
        );

        Ok(execution_time)
    }

    pub async fn rollback_migration(&self, migration: &MigrationFile) -> Result<()> {
        if let Some(down_sql) = &migration.down_sql {
            let mut tx = self.pool.begin().await?;

            sqlx::query(down_sql).execute(&mut *tx).await?;

            sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
                .bind(&migration.version)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;

            info!(
                "Rolled back migration {} ({})",
                migration.version, migration.name
            );
        } else {
            warn!(
                "No rollback SQL provided for migration {} ({})",
                migration.version, migration.name
            );
            return Err(anyhow::anyhow!(
                "No rollback SQL provided for migration {}",
                migration.version
            ));
        }

        Ok(())
    }

    pub async fn migration_exists(&self, version: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = ?")
                .bind(version)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }
}

impl MigrationRunner {
    pub fn new(migrator: DatabaseMigrator) -> Self {
        Self {
            migrator,
            migrations: Vec::new(),
        }
    }

    pub fn add_migration(&mut self, migration: MigrationFile) {
        self.migrations.push(migration);
        self.migrations.sort_by(|a, b| a.version.cmp(&b.version));
    }

    pub fn load_migrations_from_directory(&mut self, directory: &str) -> Result<()> {
        let migration_files = self.scan_migration_directory(directory)?;

        for migration_file in migration_files {
            self.add_migration(migration_file);
        }

        Ok(())
    }

    pub async fn initialize(&self) -> Result<()> {
        self.migrator.initialize().await
    }

    pub async fn migrate(&self) -> Result<Vec<String>> {
        let applied_migrations = self.migrator.get_applied_migrations().await?;
        let applied_versions: std::collections::HashSet<String> = applied_migrations
            .iter()
            .map(|m| m.version.clone())
            .collect();

        let mut applied = Vec::new();

        for migration in &self.migrations {
            if !applied_versions.contains(&migration.version) {
                if let Err(e) = self
                    .validate_migration_checksum(migration, &applied_migrations)
                    .await
                {
                    error!(
                        "Migration validation failed for {}: {}",
                        migration.version, e
                    );
                    return Err(e);
                }

                match self.migrator.apply_migration(migration).await {
                    Ok(_) => {
                        applied.push(migration.version.clone());
                        info!("Successfully applied migration {}", migration.version);
                    }
                    Err(e) => {
                        error!("Failed to apply migration {}: {}", migration.version, e);
                        return Err(e);
                    }
                }
            } else {
                info!("Skipping already applied migration {}", migration.version);
            }
        }

        if applied.is_empty() {
            info!("No migrations to apply - database is up to date");
        } else {
            info!("Applied {} migrations", applied.len());
        }

        Ok(applied)
    }

    pub async fn rollback(&self, target_version: Option<&str>) -> Result<Vec<String>> {
        let applied_migrations = self.migrator.get_applied_migrations().await?;
        let mut rolled_back = Vec::new();

        let migrations_to_rollback: Vec<_> = if let Some(target) = target_version {
            applied_migrations
                .iter()
                .rev()
                .take_while(|m| m.version.as_str() > target)
                .collect()
        } else {
            applied_migrations.iter().rev().take(1).collect()
        };

        for applied_migration in migrations_to_rollback {
            if let Some(migration_file) = self
                .migrations
                .iter()
                .find(|m| m.version == applied_migration.version)
            {
                match self.migrator.rollback_migration(migration_file).await {
                    Ok(_) => {
                        rolled_back.push(migration_file.version.clone());
                        info!(
                            "Successfully rolled back migration {}",
                            migration_file.version
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to rollback migration {}: {}",
                            migration_file.version, e
                        );
                        return Err(e);
                    }
                }
            } else {
                warn!(
                    "Migration file not found for applied migration {}",
                    applied_migration.version
                );
            }
        }

        Ok(rolled_back)
    }

    pub async fn get_migration_status(&self) -> Result<MigrationStatus> {
        let applied_migrations = self.migrator.get_applied_migrations().await?;
        let applied_versions: std::collections::HashSet<String> = applied_migrations
            .iter()
            .map(|m| m.version.clone())
            .collect();

        let pending_migrations: Vec<_> = self
            .migrations
            .iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect();

        let mut conflicts = Vec::new();

        for applied in &applied_migrations {
            if let Some(file_migration) = self
                .migrations
                .iter()
                .find(|m| m.version == applied.version)
            {
                if file_migration.checksum != applied.checksum {
                    conflicts.push(applied.version.clone());
                }
            }
        }

        Ok(MigrationStatus {
            applied: applied_migrations,
            pending: pending_migrations.into_iter().cloned().collect(),
            conflicts,
        })
    }

    async fn validate_migration_checksum(
        &self,
        migration: &MigrationFile,
        applied_migrations: &[Migration],
    ) -> Result<()> {
        for applied in applied_migrations {
            if let Some(existing_migration) = self
                .migrations
                .iter()
                .find(|m| m.version == applied.version)
            {
                if existing_migration.checksum != applied.checksum {
                    return Err(anyhow::anyhow!(
                        "Checksum mismatch for migration {}. Applied: {}, File: {}",
                        applied.version,
                        applied.checksum,
                        existing_migration.checksum
                    ));
                }
            }
        }
        Ok(())
    }

    fn scan_migration_directory(&self, directory: &str) -> Result<Vec<MigrationFile>> {
        use std::fs;
        use std::path::Path;

        let mut migrations = Vec::new();
        let path = Path::new(directory);

        if !path.exists() {
            return Ok(migrations);
        }

        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();

            if let Some(extension) = file_path.extension() {
                if extension == "sql" {
                    if let Some(file_name) = file_path.file_stem() {
                        let file_name_str = file_name.to_string_lossy();
                        if let Some(migration) =
                            self.parse_migration_file(&file_path, &file_name_str)?
                        {
                            migrations.push(migration);
                        }
                    }
                }
            }
        }

        migrations.sort_by(|a, b| a.version.cmp(&b.version));
        Ok(migrations)
    }

    fn parse_migration_file(
        &self,
        file_path: &std::path::Path,
        file_name: &str,
    ) -> Result<Option<MigrationFile>> {
        use std::fs;

        if !file_name.contains("_") {
            return Ok(None);
        }

        let parts: Vec<&str> = file_name.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Ok(None);
        }

        let version = parts[0].to_string();
        let name = parts[1].replace('_', " ");

        let content = fs::read_to_string(file_path)?;
        let checksum = self.calculate_checksum(&content);

        let (up_sql, down_sql) = if content.contains("-- DOWN") {
            let parts: Vec<&str> = content.split("-- DOWN").collect();
            if parts.len() == 2 {
                (
                    parts[0].trim().to_string(),
                    Some(parts[1].trim().to_string()),
                )
            } else {
                (content, None)
            }
        } else {
            (content, None)
        };

        Ok(Some(MigrationFile {
            version,
            name,
            up_sql,
            down_sql,
            checksum,
        }))
    }

    fn calculate_checksum(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

#[derive(Debug)]
pub struct MigrationStatus {
    pub applied: Vec<Migration>,
    pub pending: Vec<MigrationFile>,
    pub conflicts: Vec<String>,
}

impl MigrationStatus {
    pub fn is_up_to_date(&self) -> bool {
        self.pending.is_empty() && self.conflicts.is_empty()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_file_checksum() {
        let runner = MigrationRunner::new(Box::new(MockMigrator));
        let content = "CREATE TABLE test (id INTEGER PRIMARY KEY);";
        let checksum1 = runner.calculate_checksum(content);
        let checksum2 = runner.calculate_checksum(content);

        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
    }

    #[test]
    fn test_migration_file_parsing() {
        let runner = MigrationRunner::new(Box::new(MockMigrator));
        let content = r#"
            CREATE TABLE users (
                id UUID PRIMARY KEY,
                username VARCHAR(255) NOT NULL
            );

            -- DOWN
            DROP TABLE users;
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("001_create_users.sql");
        std::fs::write(&file_path, content).unwrap();

        let migration = runner
            .parse_migration_file(&file_path, "001_create_users")
            .unwrap();
        assert!(migration.is_some());

        let migration = migration.unwrap();
        assert_eq!(migration.version, "001");
        assert_eq!(migration.name, "create users");
        assert!(migration.up_sql.contains("CREATE TABLE users"));
        assert!(migration.down_sql.is_some());
        assert!(migration.down_sql.unwrap().contains("DROP TABLE users"));
    }

    struct MockMigrator;

    #[async_trait::async_trait]
    impl DatabaseMigrator for MockMigrator {
        async fn initialize(&self) -> Result<()> {
            Ok(())
        }

        async fn get_applied_migrations(&self) -> Result<Vec<Migration>> {
            Ok(vec![])
        }

        async fn apply_migration(&self, _migration: &MigrationFile) -> Result<i64> {
            Ok(100)
        }

        async fn rollback_migration(&self, _migration: &MigrationFile) -> Result<()> {
            Ok(())
        }

        async fn migration_exists(&self, _version: &str) -> Result<bool> {
            Ok(false)
        }
    }
}
