pub mod commands;
pub mod parser;
pub mod validator;

use crate::core::config::AppConfig;
use crate::core::database::connection::DatabaseManager;
use crate::utils::error::ErpResult;
pub use parser::{Cli, Commands, LogLevel, MigrateCommands};

impl Cli {
    /// CLI 실행
    pub async fn run(&self, config: AppConfig) -> ErpResult<()> {
        match &self.command {
            Some(command) => match command {
                // 마이그레이션 명령어는 자체적으로 데이터베이스를 초기화함
                Commands::Migrate(cmd) => {
                    commands::migrate::handle_migrate_command(cmd.clone(), config).await
                }
                // 다른 명령어들은 데이터베이스 초기화 필요
                _ => {
                    // 데이터베이스 초기화
                    DatabaseManager::initialize(config.database.clone()).await?;

                    match command {
                        Commands::Inventory(cmd) => {
                            commands::InventoryHandler::handle(cmd, &config).await
                        }
                        Commands::Customers(cmd) => {
                            commands::CustomerHandler::handle(cmd, &config).await
                        }
                        Commands::Sales(cmd) => commands::SalesHandler::handle(cmd, &config).await,
                        Commands::Reports(cmd) => {
                            commands::ReportsHandler::handle(cmd, &config).await
                        }
                        Commands::Config(cmd) => {
                            commands::ConfigHandler::handle(cmd, &config).await
                        }
                        Commands::Migrate(_) => unreachable!(), // 이미 위에서 처리됨
                    }
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
