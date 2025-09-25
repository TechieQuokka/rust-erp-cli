pub mod cli;
pub mod core;
pub mod modules;
pub mod utils;

use anyhow::Result;
use console::style;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub log_level: String,
    pub environment: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://erp.db".to_string(),
            log_level: "info".to_string(),
            environment: "development".to_string(),
        }
    }
}

pub fn init_logging(cli: &cli::Cli) -> Result<()> {
    let log_level = if cli.quiet {
        "warn"
    } else if cli.verbose {
        "debug"
    } else {
        "info"
    };

    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(log_level))?;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    info!("Logging initialized with level: {}", log_level);
    Ok(())
}

pub async fn init_config(config_path: Option<&str>) -> Result<AppConfig> {
    let config = match config_path {
        Some(path) => {
            info!("Loading configuration from: {}", path);
            // TODO: Implement configuration loading from file
            warn!("Custom config loading not implemented yet, using defaults");
            AppConfig::default()
        }
        None => {
            info!("Using default configuration");
            AppConfig::default()
        }
    };

    info!(
        "Configuration loaded - Environment: {}, Database: {}",
        config.environment,
        if config.database_url.contains("://") {
            &config.database_url.split("://").next().unwrap_or("unknown")
        } else {
            "unknown"
        }
    );

    Ok(config)
}

pub fn print_banner() {
    println!();
    println!(
        "{}",
        style("╔══════════════════════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("║                    🏢 ERP CLI System                      ║").cyan()
    );
    println!(
        "{}",
        style("║               High-Performance Business Management        ║").cyan()
    );
    println!(
        "{}",
        style("║                      Built with Rust                     ║").cyan()
    );
    println!(
        "{}",
        style("╚══════════════════════════════════════════════════════════╝").cyan()
    );
    println!();
}

pub fn print_success(message: &str) {
    println!("{} {}", style("✅").green(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", style("⚠️").yellow(), message);
}

pub fn print_error(message: &str) {
    println!("{} {}", style("❌").red(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ️").blue(), message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database_url, "sqlite://erp.db");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.environment, "development");
    }

    #[tokio::test]
    async fn test_init_config_default() {
        let config = init_config(None).await.unwrap();
        assert_eq!(config.environment, "development");
    }
}
