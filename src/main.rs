use anyhow::Result;
use console::style;
use erp_cli::{cli, init_config, init_logging};
use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", style("Error:").red().bold(), e);

        // Print the full error chain
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("{} {}", style("Caused by:").yellow(), err);
            source = err.source();
        }

        process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Parse command line arguments
    let cli_args = cli::parser::CliParser::parse()?;

    // Initialize logging based on verbosity
    init_logging(&cli_args)?;

    // Initialize configuration
    let _config = init_config(cli_args.config.as_deref()).await?;

    // Create command dispatcher and handle the command
    let dispatcher = cli::commands::CommandDispatcher::new();
    dispatcher.dispatch(cli_args).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_help_command() {
        // Test that help command doesn't crash
        let result = cli::parser::CliParser::parse_from(&["erp", "--help"]);
        // Help command will cause parse to exit, so we expect an error here
        assert!(result.is_err());
    }
}
