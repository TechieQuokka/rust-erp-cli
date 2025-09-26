pub mod commands;
pub mod parser;
pub mod validator;

use crate::core::config::AppConfig;
use crate::utils::error::ErpResult;
pub use parser::{Cli, Commands, LogLevel, MigrateCommands};

impl Cli {
    /// CLI 실행
    pub async fn run(&self, config: AppConfig) -> ErpResult<()> {
        match &self.command {
            Some(command) => match command {
                Commands::Inventory(cmd) => commands::InventoryHandler::handle(cmd, &config).await,
                Commands::Customers(cmd) => commands::CustomerHandler::handle(cmd, &config).await,
                Commands::Sales(cmd) => commands::SalesHandler::handle(cmd, &config).await,
                Commands::Reports(cmd) => commands::ReportsHandler::handle(cmd, &config).await,
                Commands::Config(cmd) => commands::ConfigHandler::handle(cmd, &config).await,
                Commands::Migrate(cmd) => {
                    commands::migrate::handle_migrate_command(cmd.clone(), config).await
                }
            },
            None => {
                // 기본 도움말 표시
                println!("ERP CLI - Enterprise Resource Planning System");
                println!("사용법: erp <명령어> [옵션]");
                println!("자세한 도움말은 'erp --help'를 사용하세요");
                Ok(())
            }
        }
    }
}
