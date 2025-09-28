use crate::utils::error::{ErpError, ErpResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_dir: PathBuf,
    pub schedule_cron: String,
    pub retention_days: i64,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub remote_storage_enabled: bool,
    pub remote_storage_config: Option<RemoteStorageConfig>,
    pub databases: Vec<DatabaseConfig>,
    pub file_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStorageConfig {
    pub provider: String,
    pub bucket_name: String,
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub endpoint_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: Option<String>,
    pub connection_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
    Redis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub backup_type: BackupType,
    pub size_bytes: u64,
    pub compressed: bool,
    pub encrypted: bool,
    pub file_path: PathBuf,
    pub remote_path: Option<String>,
    pub checksum: String,
    pub databases: Vec<String>,
    pub file_count: usize,
    pub status: BackupStatus,
    pub error_message: Option<String>,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Full,
    Incremental,
    Database,
    Files,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupStatus {
    InProgress,
    Completed,
    Failed,
    Verifying,
    Verified,
}

#[derive(Debug, Clone)]
pub struct RestoreOptions {
    pub backup_id: Uuid,
    pub target_directory: Option<PathBuf>,
    pub selective_restore: Option<Vec<String>>,
    pub verify_before_restore: bool,
    pub force_overwrite: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup_dir: PathBuf::from("./backups"),
            schedule_cron: "0 2 * * *".to_string(), // Daily at 2 AM
            retention_days: 30,
            compression_enabled: true,
            encryption_enabled: false,
            remote_storage_enabled: false,
            remote_storage_config: None,
            databases: Vec::new(),
            file_paths: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync {
    async fn store_metadata(&self, metadata: &BackupMetadata) -> ErpResult<()>;
    async fn get_metadata(&self, backup_id: Uuid) -> ErpResult<Option<BackupMetadata>>;
    async fn list_backups(&self, backup_type: Option<BackupType>)
        -> ErpResult<Vec<BackupMetadata>>;
    async fn update_metadata(&self, metadata: &BackupMetadata) -> ErpResult<()>;
    async fn delete_metadata(&self, backup_id: Uuid) -> ErpResult<()>;
    async fn cleanup_old_backups(&self, before: DateTime<Utc>) -> ErpResult<u64>;
}

pub struct BackupService {
    config: BackupConfig,
    repository: Box<dyn BackupRepository>,
}

impl BackupService {
    pub fn new(config: BackupConfig, repository: Box<dyn BackupRepository>) -> Self {
        Self { config, repository }
    }

    pub async fn create_full_backup(&self) -> ErpResult<BackupMetadata> {
        if !self.config.enabled {
            return Err(ErpError::validation("backup", "Backup service is disabled"));
        }

        info!("Starting full backup");
        let start_time = std::time::Instant::now();

        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let backup_filename = format!(
            "full_backup_{}_{}.tar",
            timestamp.format("%Y%m%d_%H%M%S"),
            backup_id
        );
        let backup_path = self.config.backup_dir.join(&backup_filename);

        // Ensure backup directory exists
        fs::create_dir_all(&self.config.backup_dir)?;

        let mut metadata = BackupMetadata {
            id: backup_id,
            timestamp,
            backup_type: BackupType::Full,
            size_bytes: 0,
            compressed: self.config.compression_enabled,
            encrypted: self.config.encryption_enabled,
            file_path: backup_path.clone(),
            remote_path: None,
            checksum: String::new(),
            databases: Vec::new(),
            file_count: 0,
            status: BackupStatus::InProgress,
            error_message: None,
            duration_seconds: None,
        };

        // Store initial metadata
        self.repository.store_metadata(&metadata).await?;

        // Create backup
        match self
            .create_backup_archive(&backup_path, &mut metadata)
            .await
        {
            Ok(_) => {
                metadata.status = BackupStatus::Completed;
                metadata.duration_seconds = Some(start_time.elapsed().as_secs());

                // Calculate checksum
                metadata.checksum = self.calculate_file_checksum(&backup_path)?;

                info!(
                    "Full backup completed: {} ({} bytes)",
                    backup_path.display(),
                    metadata.size_bytes
                );
            }
            Err(e) => {
                error!("Full backup failed: {}", e);
                metadata.status = BackupStatus::Failed;
                metadata.error_message = Some(e.to_string());
                metadata.duration_seconds = Some(start_time.elapsed().as_secs());

                // Clean up failed backup file
                let _ = fs::remove_file(&backup_path);
            }
        }

        // Update metadata
        self.repository.update_metadata(&metadata).await?;

        // Upload to remote storage if configured
        if self.config.remote_storage_enabled && metadata.status == BackupStatus::Completed {
            if let Err(e) = self.upload_to_remote_storage(&metadata).await {
                warn!("Failed to upload backup to remote storage: {}", e);
            }
        }

        Ok(metadata)
    }

    pub async fn create_database_backup(&self, database_name: &str) -> ErpResult<BackupMetadata> {
        if !self.config.enabled {
            return Err(ErpError::validation("backup", "Backup service is disabled"));
        }

        let db_config = self
            .config
            .databases
            .iter()
            .find(|db| db.name == database_name)
            .ok_or_else(|| ErpError::not_found("database", database_name))?;

        info!("Starting database backup for: {}", database_name);
        let start_time = std::time::Instant::now();

        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let backup_filename = format!(
            "db_{}_{}.sql",
            database_name,
            timestamp.format("%Y%m%d_%H%M%S")
        );
        let backup_path = self.config.backup_dir.join(&backup_filename);

        // Ensure backup directory exists
        fs::create_dir_all(&self.config.backup_dir)?;

        let mut metadata = BackupMetadata {
            id: backup_id,
            timestamp,
            backup_type: BackupType::Database,
            size_bytes: 0,
            compressed: false,
            encrypted: false,
            file_path: backup_path.clone(),
            remote_path: None,
            checksum: String::new(),
            databases: vec![database_name.to_string()],
            file_count: 1,
            status: BackupStatus::InProgress,
            error_message: None,
            duration_seconds: None,
        };

        // Store initial metadata
        self.repository.store_metadata(&metadata).await?;

        // Create database backup
        match self.backup_database(db_config, &backup_path).await {
            Ok(size) => {
                metadata.size_bytes = size;
                metadata.status = BackupStatus::Completed;
                metadata.duration_seconds = Some(start_time.elapsed().as_secs());
                metadata.checksum = self.calculate_file_checksum(&backup_path)?;

                info!(
                    "Database backup completed: {} ({} bytes)",
                    backup_path.display(),
                    metadata.size_bytes
                );
            }
            Err(e) => {
                error!("Database backup failed: {}", e);
                metadata.status = BackupStatus::Failed;
                metadata.error_message = Some(e.to_string());
                metadata.duration_seconds = Some(start_time.elapsed().as_secs());

                // Clean up failed backup file
                let _ = fs::remove_file(&backup_path);
            }
        }

        // Update metadata
        self.repository.update_metadata(&metadata).await?;

        Ok(metadata)
    }

    pub async fn restore_backup(&self, options: RestoreOptions) -> ErpResult<()> {
        let metadata = self
            .repository
            .get_metadata(options.backup_id)
            .await?
            .ok_or_else(|| ErpError::not_found("backup", &options.backup_id.to_string()))?;

        if metadata.status != BackupStatus::Completed && metadata.status != BackupStatus::Verified {
            return Err(ErpError::validation(
                "backup",
                "Backup is not in a restorable state",
            ));
        }

        info!("Starting restore from backup: {}", metadata.id);

        // Verify backup integrity if requested
        if options.verify_before_restore {
            self.verify_backup(&metadata).await?;
        }

        match metadata.backup_type {
            BackupType::Full | BackupType::Files => {
                self.restore_file_backup(&metadata, &options).await?;
            }
            BackupType::Database => {
                self.restore_database_backup(&metadata, &options).await?;
            }
            BackupType::Incremental => {
                return Err(ErpError::not_implemented(
                    "Incremental restore not yet implemented",
                ));
            }
        }

        info!(
            "Restore completed successfully from backup: {}",
            metadata.id
        );
        Ok(())
    }

    pub async fn verify_backup(&self, metadata: &BackupMetadata) -> ErpResult<()> {
        info!("Verifying backup: {}", metadata.id);

        // Check if backup file exists
        if !metadata.file_path.exists() {
            return Err(ErpError::not_found(
                "backup_file",
                &metadata.file_path.display().to_string(),
            ));
        }

        // Verify file size
        let file_size = fs::metadata(&metadata.file_path)?.len();
        if file_size != metadata.size_bytes {
            return Err(ErpError::validation("backup", "File size mismatch"));
        }

        // Verify checksum
        let calculated_checksum = self.calculate_file_checksum(&metadata.file_path)?;
        if calculated_checksum != metadata.checksum {
            return Err(ErpError::validation(
                "backup",
                "Checksum verification failed",
            ));
        }

        // Update metadata to mark as verified
        let mut updated_metadata = metadata.clone();
        updated_metadata.status = BackupStatus::Verified;
        self.repository.update_metadata(&updated_metadata).await?;

        info!("Backup verification successful: {}", metadata.id);
        Ok(())
    }

    pub async fn list_backups(
        &self,
        backup_type: Option<BackupType>,
    ) -> ErpResult<Vec<BackupMetadata>> {
        self.repository.list_backups(backup_type).await
    }

    pub async fn delete_backup(&self, backup_id: Uuid) -> ErpResult<()> {
        let metadata = self
            .repository
            .get_metadata(backup_id)
            .await?
            .ok_or_else(|| ErpError::not_found("backup", &backup_id.to_string()))?;

        // Delete backup file
        if metadata.file_path.exists() {
            fs::remove_file(&metadata.file_path)?;
            debug!("Deleted backup file: {}", metadata.file_path.display());
        }

        // Delete from remote storage if applicable
        if let Some(ref remote_path) = metadata.remote_path {
            if let Err(e) = self.delete_from_remote_storage(remote_path).await {
                warn!("Failed to delete backup from remote storage: {}", e);
            }
        }

        // Delete metadata
        self.repository.delete_metadata(backup_id).await?;

        info!("Backup deleted: {}", backup_id);
        Ok(())
    }

    pub async fn cleanup_old_backups(&self) -> ErpResult<u64> {
        let cutoff_date = Utc::now() - Duration::days(self.config.retention_days);

        let old_backups = self
            .repository
            .list_backups(None)
            .await?
            .into_iter()
            .filter(|backup| backup.timestamp < cutoff_date)
            .collect::<Vec<_>>();

        let mut deleted_count = 0;

        for backup in old_backups {
            if let Err(e) = self.delete_backup(backup.id).await {
                warn!("Failed to delete old backup {}: {}", backup.id, e);
            } else {
                deleted_count += 1;
            }
        }

        if deleted_count > 0 {
            info!("Cleaned up {} old backups", deleted_count);
        }

        Ok(deleted_count)
    }

    async fn create_backup_archive(
        &self,
        backup_path: &Path,
        metadata: &mut BackupMetadata,
    ) -> ErpResult<()> {
        let mut file_count = 0;

        // Create tar archive
        let backup_file = File::create(backup_path)?;
        let mut ar = tar::Builder::new(backup_file);

        // Add database dumps
        for db_config in &self.config.databases {
            let db_dump_path = format!("database_{}.sql", db_config.name);
            let temp_dump = std::env::temp_dir().join(&db_dump_path);

            if let Ok(_size) = self.backup_database(db_config, &temp_dump).await {
                if let Ok(mut file) = File::open(&temp_dump) {
                    ar.append_file(&db_dump_path, &mut file)?;
                    metadata.databases.push(db_config.name.clone());
                    file_count += 1;
                }
                let _ = fs::remove_file(&temp_dump);
            }
        }

        // Add files and directories
        for file_path in &self.config.file_paths {
            if file_path.is_file() {
                let file_name = file_path
                    .file_name()
                    .ok_or_else(|| ErpError::validation("file_path", "Invalid file name"))?
                    .to_string_lossy()
                    .into_owned();

                let mut file = File::open(file_path)?;
                ar.append_file(&file_name, &mut file)?;
                file_count += 1;
            } else if file_path.is_dir() {
                ar.append_dir_all(file_path.file_name().unwrap(), file_path)?;
                file_count += self.count_files_in_dir(file_path)?;
            }
        }

        ar.finish()?;

        // Get final file size
        metadata.size_bytes = fs::metadata(backup_path)?.len();
        metadata.file_count = file_count;

        Ok(())
    }

    async fn backup_database(
        &self,
        db_config: &DatabaseConfig,
        output_path: &Path,
    ) -> ErpResult<u64> {
        match db_config.db_type {
            DatabaseType::PostgreSQL => self.backup_postgresql(db_config, output_path).await,
            DatabaseType::MySQL => self.backup_mysql(db_config, output_path).await,
            DatabaseType::SQLite => self.backup_sqlite(db_config, output_path).await,
            DatabaseType::Redis => self.backup_redis(db_config, output_path).await,
        }
    }

    async fn backup_postgresql(
        &self,
        db_config: &DatabaseConfig,
        output_path: &Path,
    ) -> ErpResult<u64> {
        let mut cmd = Command::new("pg_dump");

        cmd.arg("-h")
            .arg(&db_config.host)
            .arg("-p")
            .arg(db_config.port.to_string())
            .arg("-U")
            .arg(&db_config.username)
            .arg("-d")
            .arg(&db_config.database_name)
            .arg("-f")
            .arg(output_path)
            .arg("--verbose")
            .arg("--no-password");

        if let Some(ref password) = db_config.password {
            cmd.env("PGPASSWORD", password);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!("pg_dump failed: {}", error)));
        }

        let size = fs::metadata(output_path)?.len();
        debug!("PostgreSQL backup completed: {} bytes", size);
        Ok(size)
    }

    async fn backup_mysql(&self, db_config: &DatabaseConfig, output_path: &Path) -> ErpResult<u64> {
        let mut cmd = Command::new("mysqldump");

        cmd.arg("-h")
            .arg(&db_config.host)
            .arg("-P")
            .arg(db_config.port.to_string())
            .arg("-u")
            .arg(&db_config.username);

        if let Some(ref password) = db_config.password {
            cmd.arg(format!("-p{}", password));
        }

        cmd.arg(&db_config.database_name);

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!("mysqldump failed: {}", error)));
        }

        fs::write(output_path, &output.stdout)?;
        let size = output.stdout.len() as u64;
        debug!("MySQL backup completed: {} bytes", size);
        Ok(size)
    }

    async fn backup_sqlite(
        &self,
        db_config: &DatabaseConfig,
        output_path: &Path,
    ) -> ErpResult<u64> {
        let connection_string = db_config
            .connection_string
            .as_ref()
            .ok_or_else(|| ErpError::validation("sqlite", "Connection string required"))?;

        let db_path = Path::new(connection_string);

        if !db_path.exists() {
            return Err(ErpError::not_found(
                "sqlite_database",
                &db_path.display().to_string(),
            ));
        }

        // For SQLite, we can simply copy the database file
        fs::copy(db_path, output_path)?;
        let size = fs::metadata(output_path)?.len();
        debug!("SQLite backup completed: {} bytes", size);
        Ok(size)
    }

    async fn backup_redis(
        &self,
        _db_config: &DatabaseConfig,
        output_path: &Path,
    ) -> ErpResult<u64> {
        // Redis backup would typically use BGSAVE or similar
        // For now, we'll create a placeholder
        let backup_content =
            "# Redis backup placeholder\n# Implement using redis-cli or direct connection\n";
        fs::write(output_path, backup_content)?;
        let size = backup_content.len() as u64;
        debug!("Redis backup placeholder created: {} bytes", size);
        Ok(size)
    }

    async fn restore_file_backup(
        &self,
        metadata: &BackupMetadata,
        options: &RestoreOptions,
    ) -> ErpResult<()> {
        let default_restore_dir = PathBuf::from("./restore");
        let target_dir = options
            .target_directory
            .as_ref()
            .unwrap_or(&default_restore_dir);

        fs::create_dir_all(target_dir)?;

        // Extract tar archive
        let backup_file = File::open(&metadata.file_path)?;
        let mut ar = tar::Archive::new(backup_file);

        if let Some(ref selective) = options.selective_restore {
            // Selective restore
            for entry in ar.entries()? {
                let mut entry = entry?;
                let path = entry.path()?.to_path_buf();
                let path_str = path.to_string_lossy();

                if selective.iter().any(|pattern| path_str.contains(pattern)) {
                    entry.unpack_in(target_dir)?;
                    debug!("Restored file: {}", path_str);
                }
            }
        } else {
            // Full restore
            ar.unpack(target_dir)?;
            debug!("Full restore completed to: {}", target_dir.display());
        }

        Ok(())
    }

    async fn restore_database_backup(
        &self,
        metadata: &BackupMetadata,
        options: &RestoreOptions,
    ) -> ErpResult<()> {
        if metadata.databases.is_empty() {
            return Err(ErpError::validation(
                "backup",
                "No database information in backup",
            ));
        }

        let database_name = &metadata.databases[0];
        let db_config = self
            .config
            .databases
            .iter()
            .find(|db| db.name == *database_name)
            .ok_or_else(|| ErpError::not_found("database", database_name))?;

        match db_config.db_type {
            DatabaseType::PostgreSQL => {
                self.restore_postgresql(db_config, &metadata.file_path)
                    .await
            }
            DatabaseType::MySQL => self.restore_mysql(db_config, &metadata.file_path).await,
            DatabaseType::SQLite => {
                self.restore_sqlite(db_config, &metadata.file_path, options)
                    .await
            }
            DatabaseType::Redis => self.restore_redis(db_config, &metadata.file_path).await,
        }
    }

    async fn restore_postgresql(
        &self,
        db_config: &DatabaseConfig,
        backup_path: &Path,
    ) -> ErpResult<()> {
        let mut cmd = Command::new("psql");

        cmd.arg("-h")
            .arg(&db_config.host)
            .arg("-p")
            .arg(db_config.port.to_string())
            .arg("-U")
            .arg(&db_config.username)
            .arg("-d")
            .arg(&db_config.database_name)
            .arg("-f")
            .arg(backup_path);

        if let Some(ref password) = db_config.password {
            cmd.env("PGPASSWORD", password);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!(
                "psql restore failed: {}",
                error
            )));
        }

        debug!("PostgreSQL restore completed");
        Ok(())
    }

    async fn restore_mysql(&self, db_config: &DatabaseConfig, backup_path: &Path) -> ErpResult<()> {
        let mut cmd = Command::new("mysql");

        cmd.arg("-h")
            .arg(&db_config.host)
            .arg("-P")
            .arg(db_config.port.to_string())
            .arg("-u")
            .arg(&db_config.username);

        if let Some(ref password) = db_config.password {
            cmd.arg(format!("-p{}", password));
        }

        cmd.arg(&db_config.database_name);

        let backup_content = fs::read_to_string(backup_path)?;

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(backup_content.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!(
                "mysql restore failed: {}",
                error
            )));
        }

        debug!("MySQL restore completed");
        Ok(())
    }

    async fn restore_sqlite(
        &self,
        db_config: &DatabaseConfig,
        backup_path: &Path,
        options: &RestoreOptions,
    ) -> ErpResult<()> {
        let target_path = if let Some(ref target) = options.target_directory {
            target.join(format!("{}.db", db_config.database_name))
        } else {
            let connection_string = db_config
                .connection_string
                .as_ref()
                .ok_or_else(|| ErpError::validation("sqlite", "Connection string required"))?;
            Path::new(connection_string).to_path_buf()
        };

        if target_path.exists() && !options.force_overwrite {
            return Err(ErpError::validation(
                "restore",
                "Target database exists and force_overwrite is false",
            ));
        }

        fs::copy(backup_path, &target_path)?;
        debug!("SQLite restore completed to: {}", target_path.display());
        Ok(())
    }

    async fn restore_redis(
        &self,
        _db_config: &DatabaseConfig,
        _backup_path: &Path,
    ) -> ErpResult<()> {
        // Redis restore would typically use redis-cli or direct connection
        debug!("Redis restore - placeholder implementation");
        Ok(())
    }

    async fn upload_to_remote_storage(&self, metadata: &BackupMetadata) -> ErpResult<()> {
        if let Some(ref remote_config) = self.config.remote_storage_config {
            info!("Uploading backup to remote storage: {}", metadata.id);

            let remote_path = format!(
                "{}/backup_{}.tar",
                metadata.timestamp.format("%Y/%m/%d"),
                metadata.id
            );

            // Placeholder for actual remote storage implementation
            // This would typically use AWS S3, Azure Blob, or similar
            debug!(
                "Remote upload placeholder for provider: {}",
                remote_config.provider
            );
            debug!(
                "Would upload to: {}/{}",
                remote_config.bucket_name, remote_path
            );

            // Update metadata with remote path
            let mut updated_metadata = metadata.clone();
            updated_metadata.remote_path = Some(remote_path);
            self.repository.update_metadata(&updated_metadata).await?;
        }

        Ok(())
    }

    async fn delete_from_remote_storage(&self, _remote_path: &str) -> ErpResult<()> {
        // Placeholder for remote storage deletion
        debug!("Remote deletion placeholder");
        Ok(())
    }

    fn calculate_file_checksum(&self, file_path: &Path) -> ErpResult<String> {
        use sha2::{Digest, Sha256};

        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn count_files_in_dir(&self, dir_path: &Path) -> ErpResult<usize> {
        let mut count = 0;

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += self.count_files_in_dir(&path)?;
            }
        }

        Ok(count)
    }
}

// Mock implementation for testing
#[derive(Debug, Clone)]
pub struct MockBackupRepository {
    backups: std::sync::Arc<std::sync::Mutex<Vec<BackupMetadata>>>,
}

impl Default for MockBackupRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackupRepository {
    pub fn new() -> Self {
        Self {
            backups: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl BackupRepository for MockBackupRepository {
    async fn store_metadata(&self, metadata: &BackupMetadata) -> ErpResult<()> {
        let mut backups = self.backups.lock().unwrap();
        backups.push(metadata.clone());
        Ok(())
    }

    async fn get_metadata(&self, backup_id: Uuid) -> ErpResult<Option<BackupMetadata>> {
        let backups = self.backups.lock().unwrap();
        Ok(backups.iter().find(|b| b.id == backup_id).cloned())
    }

    async fn list_backups(
        &self,
        backup_type: Option<BackupType>,
    ) -> ErpResult<Vec<BackupMetadata>> {
        let backups = self.backups.lock().unwrap();
        if let Some(filter_type) = backup_type {
            Ok(backups
                .iter()
                .filter(|b| b.backup_type == filter_type)
                .cloned()
                .collect())
        } else {
            Ok(backups.clone())
        }
    }

    async fn update_metadata(&self, metadata: &BackupMetadata) -> ErpResult<()> {
        let mut backups = self.backups.lock().unwrap();
        if let Some(existing) = backups.iter_mut().find(|b| b.id == metadata.id) {
            *existing = metadata.clone();
        }
        Ok(())
    }

    async fn delete_metadata(&self, backup_id: Uuid) -> ErpResult<()> {
        let mut backups = self.backups.lock().unwrap();
        backups.retain(|b| b.id != backup_id);
        Ok(())
    }

    async fn cleanup_old_backups(&self, before: DateTime<Utc>) -> ErpResult<u64> {
        let mut backups = self.backups.lock().unwrap();
        let initial_len = backups.len();
        backups.retain(|b| b.timestamp > before);
        Ok((initial_len - backups.len()) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert!(config.enabled);
        assert_eq!(config.retention_days, 30);
        assert!(config.compression_enabled);
    }

    #[test]
    fn test_backup_metadata_creation() {
        let metadata = BackupMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            backup_type: BackupType::Full,
            size_bytes: 1024,
            compressed: true,
            encrypted: false,
            file_path: PathBuf::from("/tmp/backup.tar"),
            remote_path: None,
            checksum: "abc123".to_string(),
            databases: vec!["test_db".to_string()],
            file_count: 5,
            status: BackupStatus::Completed,
            error_message: None,
            duration_seconds: Some(30),
        };

        assert_eq!(metadata.backup_type, BackupType::Full);
        assert_eq!(metadata.status, BackupStatus::Completed);
        assert_eq!(metadata.size_bytes, 1024);
    }

    #[tokio::test]
    async fn test_mock_backup_repository() {
        let repository = MockBackupRepository::new();

        let metadata = BackupMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            backup_type: BackupType::Database,
            size_bytes: 512,
            compressed: false,
            encrypted: false,
            file_path: PathBuf::from("/tmp/db_backup.sql"),
            remote_path: None,
            checksum: "def456".to_string(),
            databases: vec!["postgres".to_string()],
            file_count: 1,
            status: BackupStatus::Completed,
            error_message: None,
            duration_seconds: Some(15),
        };

        // Store metadata
        repository.store_metadata(&metadata).await.unwrap();

        // Retrieve metadata
        let retrieved = repository.get_metadata(metadata.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, metadata.id);

        // List backups
        let backups = repository
            .list_backups(Some(BackupType::Database))
            .await
            .unwrap();
        assert_eq!(backups.len(), 1);

        // Delete metadata
        repository.delete_metadata(metadata.id).await.unwrap();
        let deleted = repository.get_metadata(metadata.id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_backup_service_creation() {
        let config = BackupConfig::default();
        let repository = Box::new(MockBackupRepository::new());
        let service = BackupService::new(config, repository);

        let backups = service.list_backups(None).await.unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn test_database_config() {
        let db_config = DatabaseConfig {
            name: "test_db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database_name: "testdb".to_string(),
            username: "postgres".to_string(),
            password: Some("password".to_string()),
            connection_string: None,
        };

        assert_eq!(db_config.name, "test_db");
        assert_eq!(db_config.port, 5432);
        assert!(matches!(db_config.db_type, DatabaseType::PostgreSQL));
    }

    #[test]
    fn test_restore_options() {
        let options = RestoreOptions {
            backup_id: Uuid::new_v4(),
            target_directory: Some(PathBuf::from("/tmp/restore")),
            selective_restore: Some(vec!["database_".to_string()]),
            verify_before_restore: true,
            force_overwrite: false,
        };

        assert!(options.verify_before_restore);
        assert!(!options.force_overwrite);
        assert!(options.selective_restore.is_some());
    }

    #[tokio::test]
    async fn test_backup_service_disabled() {
        let mut config = BackupConfig::default();
        config.enabled = false;

        let repository = Box::new(MockBackupRepository::new());
        let service = BackupService::new(config, repository);

        let result = service.create_full_backup().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_file_checksum_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let config = BackupConfig::default();
        let repository = Box::new(MockBackupRepository::new());
        let service = BackupService::new(config, repository);

        let checksum = service.calculate_file_checksum(&test_file).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA256 hash length in hex
    }
}
