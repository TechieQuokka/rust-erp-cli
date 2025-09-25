pub mod config;
pub mod customers;
pub mod inventory;
pub mod reports;
pub mod sales;

use crate::cli::{Cli, Commands};
use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait CommandHandler {
    async fn handle(&self, cli: &Cli) -> Result<()>;
}

#[derive(Default)]
pub struct CommandDispatcher;

impl CommandDispatcher {
    pub fn new() -> Self {
        Self
    }

    pub async fn dispatch(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Auth { command } => {
                // TODO: Implement auth command handling
                println!("Auth command: {:?}", command);
                Ok(())
            }
            Commands::Inventory { command } => {
                let handler = inventory::InventoryCommandHandler::new();
                handler.handle_inventory_command(command).await
            }
            Commands::Customers { command } => {
                let handler = customers::CustomerCommandHandler::new();
                handler.handle_customer_command(command).await
            }
            Commands::Sales { command } => {
                let handler = sales::SalesCommandHandler::new();
                handler.handle_sales_command(command).await
            }
            Commands::Reports { command } => {
                let handler = reports::ReportCommandHandler::new();
                handler.handle_report_command(command).await
            }
            Commands::Config { command } => {
                let handler = config::ConfigCommandHandler::new();
                handler.handle_config_command(command).await
            }
            Commands::Setup {
                init_db,
                sample_data,
            } => {
                println!(
                    "Setup command - init_db: {}, sample_data: {}",
                    init_db, sample_data
                );
                if init_db {
                    println!("Initializing database...");
                    // TODO: Implement database initialization
                }
                if sample_data {
                    println!("Creating sample data...");
                    // TODO: Implement sample data creation
                }
                Ok(())
            }
        }
    }
}
