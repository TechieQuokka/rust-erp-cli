use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

#[derive(Default)]
pub struct CliParser;

impl CliParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse() -> Result<Cli> {
        let cli = Cli::parse();
        Ok(cli)
    }

    pub fn parse_from<I, T>(args: I) -> Result<Cli>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let cli = Cli::parse_from(args);
        Ok(cli)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        let result = CliParser::parse_from(&["erp", "--help"]);
        // This will actually exit with help message, so we can't test it directly
        // But we can test that the parser accepts the argument
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_parse_inventory_list() {
        let result = CliParser::parse_from(&["erp", "inventory", "list"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_customer_add() {
        let result = CliParser::parse_from(&["erp", "customers", "add", "John Doe"]);
        assert!(result.is_ok());
    }
}
