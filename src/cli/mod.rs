pub mod commands;
pub mod parser;
pub mod validator;

use clap::{Parser, Subcommand};
use std::fmt;

#[derive(Parser)]
#[command(name = "erp")]
#[command(about = "A high-performance modular ERP CLI system built with Rust")]
#[command(version = "0.1.0")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    #[arg(long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Authentication and user management")]
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    #[command(about = "Inventory management operations")]
    Inventory {
        #[command(subcommand)]
        command: InventoryCommands,
    },

    #[command(about = "Customer management operations")]
    Customers {
        #[command(subcommand)]
        command: CustomerCommands,
    },

    #[command(about = "Sales and order management")]
    Sales {
        #[command(subcommand)]
        command: SalesCommands,
    },

    #[command(about = "Generate various reports")]
    Reports {
        #[command(subcommand)]
        command: ReportCommands,
    },

    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    #[command(about = "System setup and initialization")]
    Setup {
        #[arg(long, help = "Initialize database with default schema")]
        init_db: bool,

        #[arg(long, help = "Create sample data for testing")]
        sample_data: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    #[command(about = "Register a new user")]
    Register {
        #[arg(short, long, help = "Username")]
        username: String,

        #[arg(short, long, help = "Email address")]
        email: String,

        #[arg(short, long, help = "User role (admin, manager, user, readonly)")]
        role: Option<String>,
    },

    #[command(about = "Login to the system")]
    Login {
        #[arg(short, long, help = "Username")]
        username: String,
    },

    #[command(about = "Logout from the system")]
    Logout,

    #[command(about = "Change password")]
    ChangePassword {
        #[arg(short, long, help = "Username")]
        username: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum InventoryCommands {
    #[command(about = "Add a new product to inventory")]
    Add {
        #[arg(help = "Product name")]
        name: String,

        #[arg(long, help = "Initial quantity")]
        quantity: i32,

        #[arg(short, long, help = "Unit price")]
        price: f64,

        #[arg(short, long, help = "Product category")]
        category: String,

        #[arg(short, long, help = "Product description")]
        description: Option<String>,

        #[arg(short, long, help = "SKU (Stock Keeping Unit)")]
        sku: Option<String>,
    },

    #[command(about = "List products in inventory")]
    List {
        #[arg(short, long, help = "Show only low-stock items")]
        low_stock: bool,

        #[arg(short, long, help = "Filter by category")]
        category: Option<String>,

        #[arg(short, long, help = "Search by name")]
        search: Option<String>,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,

        #[arg(long, help = "Number of items per page", default_value = "10")]
        limit: usize,

        #[arg(long, help = "Page number", default_value = "1")]
        page: usize,
    },

    #[command(about = "Update product information")]
    Update {
        #[arg(help = "Product ID")]
        id: String,

        #[arg(long, help = "New quantity")]
        quantity: Option<i32>,

        #[arg(short, long, help = "New unit price")]
        price: Option<f64>,

        #[arg(short = 'n', long, help = "New product name")]
        name: Option<String>,

        #[arg(short, long, help = "New category")]
        category: Option<String>,

        #[arg(short, long, help = "New description")]
        description: Option<String>,
    },

    #[command(about = "Remove a product from inventory")]
    Remove {
        #[arg(help = "Product ID")]
        id: String,

        #[arg(short, long, help = "Force removal without confirmation")]
        force: bool,
    },

    #[command(about = "Search products")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum CustomerCommands {
    #[command(about = "Add a new customer")]
    Add {
        #[arg(help = "Customer name")]
        name: String,

        #[arg(short, long, help = "Email address")]
        email: Option<String>,

        #[arg(short, long, help = "Phone number")]
        phone: Option<String>,

        #[arg(short, long, help = "Address")]
        address: Option<String>,
    },

    #[command(about = "List customers")]
    List {
        #[arg(short, long, help = "Search by name")]
        search: Option<String>,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,

        #[arg(long, help = "Number of items per page", default_value = "10")]
        limit: usize,

        #[arg(long, help = "Page number", default_value = "1")]
        page: usize,
    },

    #[command(about = "Update customer information")]
    Update {
        #[arg(help = "Customer ID")]
        id: String,

        #[arg(short = 'n', long, help = "New customer name")]
        name: Option<String>,

        #[arg(short, long, help = "New email address")]
        email: Option<String>,

        #[arg(short, long, help = "New phone number")]
        phone: Option<String>,

        #[arg(short, long, help = "New address")]
        address: Option<String>,
    },

    #[command(about = "Delete a customer")]
    Delete {
        #[arg(help = "Customer ID")]
        id: String,

        #[arg(short, long, help = "Force deletion without confirmation")]
        force: bool,
    },

    #[command(about = "Search customers")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SalesCommands {
    #[command(about = "Create a new order")]
    CreateOrder {
        #[arg(long, help = "Customer ID")]
        customer: String,

        #[arg(long, help = "Product ID")]
        product: String,

        #[arg(long, help = "Quantity to order")]
        quantity: i32,

        #[arg(long, help = "Custom unit price (overrides product price)")]
        unit_price: Option<f64>,
    },

    #[command(about = "List orders")]
    ListOrders {
        #[arg(short, long, help = "Filter by status")]
        status: Option<String>,

        #[arg(long, help = "Date range (YYYY-MM-DD,YYYY-MM-DD)")]
        date_range: Option<String>,

        #[arg(short, long, help = "Customer ID")]
        customer: Option<String>,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,

        #[arg(long, help = "Number of items per page", default_value = "10")]
        limit: usize,

        #[arg(long, help = "Page number", default_value = "1")]
        page: usize,
    },

    #[command(about = "Update order status")]
    UpdateOrder {
        #[arg(help = "Order ID")]
        id: String,

        #[arg(
            short,
            long,
            help = "New status (pending, processing, shipped, delivered, cancelled)"
        )]
        status: String,
    },

    #[command(about = "Generate invoice for an order")]
    GenerateInvoice {
        #[arg(help = "Order ID")]
        order_id: String,

        #[arg(long, help = "Output format (pdf, json)", default_value = "pdf")]
        format: String,

        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ReportCommands {
    #[command(about = "Generate sales summary report")]
    SalesSummary {
        #[arg(
            short,
            long,
            help = "Report period (daily, weekly, monthly, quarterly, yearly)",
            default_value = "monthly"
        )]
        period: String,

        #[arg(short, long, help = "Year (YYYY)", default_value = "2024")]
        year: String,

        #[arg(short, long, help = "Month (1-12) for monthly reports")]
        month: Option<u32>,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,
    },

    #[command(about = "Generate inventory status report")]
    InventoryStatus {
        #[arg(long, help = "Include only low stock items")]
        low_stock_only: bool,

        #[arg(short, long, help = "Filter by category")]
        category: Option<String>,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,
    },

    #[command(about = "Generate customer analysis report")]
    CustomerAnalysis {
        #[arg(long, help = "Show top N customers", default_value = "10")]
        top: usize,

        #[arg(long, help = "Analysis period in days", default_value = "30")]
        days: u32,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,
    },

    #[command(about = "Generate financial overview")]
    FinancialOverview {
        #[arg(short, long, help = "Quarter (Q1, Q2, Q3, Q4)", default_value = "Q1")]
        quarter: String,

        #[arg(short, long, help = "Year (YYYY)", default_value = "2024")]
        year: String,

        #[arg(
            long,
            help = "Output format (table, json, csv)",
            default_value = "table"
        )]
        format: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Show current configuration")]
    Show,

    #[command(about = "Set a configuration value")]
    Set {
        #[arg(help = "Configuration key")]
        key: String,

        #[arg(help = "Configuration value")]
        value: String,
    },

    #[command(about = "Get a configuration value")]
    Get {
        #[arg(help = "Configuration key")]
        key: String,
    },

    #[command(about = "Reset configuration to defaults")]
    Reset {
        #[arg(short, long, help = "Force reset without confirmation")]
        force: bool,
    },
}

pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

impl From<&str> for OutputFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "csv" => OutputFormat::Csv,
            _ => OutputFormat::Table,
        }
    }
}
