use crate::cli::parser::MigrateCommands;
use crate::core::config::AppConfig;
use crate::core::database::connection::DatabaseManager;
use crate::core::database::migration::{DatabaseMigrator, MigrationRunner, PostgresMigrator};
use crate::utils::error::{ErpError, ErpResult};
use chrono::{DateTime, Local};
use comfy_table::{Cell, Color, Table};
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

pub async fn handle_migrate_command(command: MigrateCommands, config: AppConfig) -> ErpResult<()> {
    match command {
        MigrateCommands::Init { force } => init_database(config, force).await,
        MigrateCommands::Up { target, dir } => run_migrations(config, target, dir).await,
        MigrateCommands::Down { target, dir } => rollback_migrations(config, target, dir).await,
        MigrateCommands::Status { dir } => show_migration_status(config, dir).await,
        MigrateCommands::Generate { name, dir } => generate_migration(name, dir).await,
        MigrateCommands::Test => test_database_connection(config).await,
    }
}

async fn init_database(config: AppConfig, force: bool) -> ErpResult<()> {
    info!("Initializing database...");

    // Initialize database manager
    DatabaseManager::initialize(config.database.clone()).await?;
    let connection = DatabaseManager::get_connection().await?;
    let migrator = create_migrator(&connection).await?;

    if force {
        warn!("Force flag detected - this will recreate migration schema");
    }

    migrator.initialize().await.map_err(|e| {
        error!("Failed to initialize database: {}", e);
        ErpError::internal(format!("Database initialization failed: {}", e))
    })?;

    println!("✅ Database initialized successfully");
    info!("Database migration schema created");
    Ok(())
}

async fn run_migrations(config: AppConfig, _target: Option<String>, dir: String) -> ErpResult<()> {
    info!("Running database migrations...");

    DatabaseManager::initialize(config.database.clone()).await?;
    let connection = DatabaseManager::get_connection().await?;
    let migrator = create_migrator(&connection).await?;
    let mut runner = MigrationRunner::new(migrator);

    // Load migration files from directory
    runner.load_migrations_from_directory(&dir).map_err(|e| {
        error!("Failed to load migration files: {}", e);
        ErpError::io(format!("Migration loading failed: {}", e))
    })?;

    // Initialize migration schema if needed
    runner.initialize().await.map_err(|e| {
        error!("Failed to initialize migration schema: {}", e);
        ErpError::internal(format!("Migration schema initialization failed: {}", e))
    })?;

    // Run migrations
    let applied_migrations = runner.migrate().await.map_err(|e| {
        error!("Migration execution failed: {}", e);
        ErpError::database(format!("Migration failed: {}", e))
    })?;

    if applied_migrations.is_empty() {
        println!("✅ Database is already up to date");
    } else {
        println!("✅ Applied {} migration(s):", applied_migrations.len());
        for migration in applied_migrations {
            println!("  - {}", migration);
        }
    }

    info!("Migrations completed successfully");
    Ok(())
}

async fn rollback_migrations(
    config: AppConfig,
    target: Option<String>,
    dir: String,
) -> ErpResult<()> {
    info!("Rolling back database migrations...");

    DatabaseManager::initialize(config.database.clone()).await?;
    let connection = DatabaseManager::get_connection().await?;
    let migrator = create_migrator(&connection).await?;
    let mut runner = MigrationRunner::new(migrator);

    // Load migration files
    runner.load_migrations_from_directory(&dir).map_err(|e| {
        error!("Failed to load migration files: {}", e);
        ErpError::io(format!("Migration loading failed: {}", e))
    })?;

    // Rollback migrations
    let rolled_back = runner.rollback(target.as_deref()).await.map_err(|e| {
        error!("Migration rollback failed: {}", e);
        ErpError::database(format!("Rollback failed: {}", e))
    })?;

    if rolled_back.is_empty() {
        println!("✅ No migrations to rollback");
    } else {
        println!("✅ Rolled back {} migration(s):", rolled_back.len());
        for migration in rolled_back {
            println!("  - {}", migration);
        }
    }

    info!("Migration rollback completed successfully");
    Ok(())
}

async fn show_migration_status(config: AppConfig, dir: String) -> ErpResult<()> {
    info!("Checking migration status...");

    DatabaseManager::initialize(config.database.clone()).await?;
    let connection = DatabaseManager::get_connection().await?;
    let migrator = create_migrator(&connection).await?;
    let mut runner = MigrationRunner::new(migrator);

    // Load migration files
    runner.load_migrations_from_directory(&dir).map_err(|e| {
        error!("Failed to load migration files: {}", e);
        ErpError::io(format!("Migration loading failed: {}", e))
    })?;

    // Get migration status
    let status = runner.get_migration_status().await.map_err(|e| {
        error!("Failed to get migration status: {}", e);
        ErpError::database(format!("Status check failed: {}", e))
    })?;

    // Display status table
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Version").fg(Color::Blue),
        Cell::new("Name").fg(Color::Blue),
        Cell::new("Status").fg(Color::Blue),
        Cell::new("Executed At").fg(Color::Blue),
    ]);

    // Show applied migrations
    for migration in &status.applied {
        table.add_row(vec![
            Cell::new(&migration.version),
            Cell::new(&migration.name),
            Cell::new("Applied").fg(Color::Green),
            Cell::new(format_datetime(&migration.executed_at)),
        ]);
    }

    // Show pending migrations
    for migration in &status.pending {
        table.add_row(vec![
            Cell::new(&migration.version),
            Cell::new(&migration.name),
            Cell::new("Pending").fg(Color::Yellow),
            Cell::new("-"),
        ]);
    }

    println!("Database Migration Status:");
    println!("{}", table);

    // Show summary
    println!("\nSummary:");
    println!("  Applied migrations: {}", status.applied_count());
    println!("  Pending migrations: {}", status.pending_count());

    if status.has_conflicts() {
        println!("  ⚠️  Conflicts detected: {}", status.conflicts.len());
        for conflict in &status.conflicts {
            println!("    - {}", conflict);
        }
    }

    if status.is_up_to_date() {
        println!("  ✅ Database is up to date");
    } else {
        println!("  ⚠️  Database needs migration");
    }

    Ok(())
}

async fn generate_migration(name: String, dir: String) -> ErpResult<()> {
    info!("Generating new migration: {}", name);

    // Create migrations directory if it doesn't exist
    let migrations_path = Path::new(&dir);
    if !migrations_path.exists() {
        fs::create_dir_all(migrations_path).map_err(|e| {
            error!("Failed to create migrations directory: {}", e);
            ErpError::io(format!("Directory creation failed: {}", e))
        })?;
    }

    // Generate version number based on current timestamp
    let _timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let version = format!("{:03}", get_next_migration_version(&dir).unwrap_or(1));
    let filename = format!("{}_{}.sql", version, name.replace(" ", "_").to_lowercase());
    let filepath = migrations_path.join(&filename);

    // Create migration template
    let template = format!(
        r#"-- Migration: {}
-- Version: {}
-- Description: {}

-- Add your migration SQL here


-- DOWN
-- Add rollback SQL here (optional)

"#,
        name, version, name
    );

    fs::write(&filepath, template).map_err(|e| {
        error!("Failed to write migration file: {}", e);
        ErpError::io(format!("Migration file creation failed: {}", e))
    })?;

    println!("✅ Created migration file: {}", filepath.display());
    println!("📝 Edit the file to add your migration SQL");

    info!("Migration file generated: {}", filepath.display());
    Ok(())
}

async fn test_database_connection(config: AppConfig) -> ErpResult<()> {
    info!("Testing database connection...");

    match DatabaseManager::initialize(config.database.clone()).await {
        Ok(_) => {
            println!("✅ Database connection successful");

            // Test a simple query using the health check
            match DatabaseManager::health_check().await {
                Ok(_) => println!("✅ Database query test passed"),
                Err(e) => {
                    error!("Database query test failed: {}", e);
                    return Err(ErpError::database(format!("Query test failed: {}", e)));
                }
            }
        }
        Err(e) => {
            error!("Database connection failed: {}", e);
            println!("❌ Database connection failed: {}", e);
            return Err(e);
        }
    }

    info!("Database connection test completed successfully");
    Ok(())
}

async fn create_migrator(
    connection: &std::sync::Arc<crate::core::database::connection::DatabaseConnection>,
) -> ErpResult<DatabaseMigrator> {
    // Currently the system only supports PostgreSQL
    Ok(DatabaseMigrator::Postgres(PostgresMigrator::new(
        connection.pool().clone(),
    )))
}

fn get_next_migration_version(dir: &str) -> Option<u32> {
    let path = Path::new(dir);
    if !path.exists() {
        return Some(1);
    }

    let mut max_version = 0u32;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".sql") {
                    if let Some(version_str) = file_name.split('_').next() {
                        if let Ok(version) = version_str.parse::<u32>() {
                            max_version = max_version.max(version);
                        }
                    }
                }
            }
        }
    }

    Some(max_version + 1)
}

fn format_datetime(dt: &DateTime<chrono::Utc>) -> String {
    let local_dt: DateTime<Local> = DateTime::from(*dt);
    local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_migration_version() {
        // Test with non-existent directory
        let version = get_next_migration_version("/non/existent/path");
        assert_eq!(version, Some(1));
    }

    #[test]
    fn test_format_datetime() {
        let dt = chrono::Utc::now();
        let formatted = format_datetime(&dt);

        // Check the format structure instead of exact date match
        // Format should be "YYYY-MM-DD HH:MM:SS"
        assert_eq!(formatted.len(), 19);
        assert_eq!(formatted.chars().nth(4), Some('-'));
        assert_eq!(formatted.chars().nth(7), Some('-'));
        assert_eq!(formatted.chars().nth(10), Some(' '));
        assert_eq!(formatted.chars().nth(13), Some(':'));
        assert_eq!(formatted.chars().nth(16), Some(':'));

        // Verify it's a valid date format by parsing it back
        use chrono::NaiveDateTime;
        assert!(NaiveDateTime::parse_from_str(&formatted, "%Y-%m-%d %H:%M:%S").is_ok());
    }
}
