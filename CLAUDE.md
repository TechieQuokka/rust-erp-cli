# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based modular ERP (Enterprise Resource Planning) CLI system following a 4-layer architecture:

1. **CLI Interface Layer** - Command parsing, validation, and user interaction
2. **Business Logic Layer** - Core business modules (inventory, sales, customers, reports, config)
3. **Core Services Layer** - Authentication, database, configuration, and logging services
4. **Data Layer** - PostgreSQL (production), SQLite (development), Redis (caching)

## Commands for Development

### Build and Run
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run the CLI
./target/release/erp --help
cargo run -- --help
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with nextest (if available)
cargo nextest run

# Run specific module tests
cargo test inventory

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Code Quality
```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings

# Check code without building
cargo check
```

### Database Management
```bash
# Initialize database
cargo run -- setup --init-db

# Run migrations (when implemented)
cargo run -- migrate
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Build docs
cargo doc --no-deps
```

## Architecture Overview

### Module Structure
- `src/cli/` - CLI command definitions, parsing, and validation
- `src/modules/` - Business logic modules:
  - `inventory/` - Product and stock management
  - `sales/` - Order processing and invoicing
  - `customers/` - Customer relationship management
  - `reports/` - Analytics and reporting
  - `config/` - System configuration
- `src/core/` - Core services:
  - `auth/` - JWT authentication and RBAC
  - `database/` - Database connection and models
  - `config/` - Configuration loading
  - `logging/` - Structured logging
- `src/utils/` - Shared utilities (error handling, crypto, validation)

### Key Design Patterns
- **Repository Pattern**: Each module has a repository for data access
- **Service Layer**: Business logic separated from data access
- **Command Pattern**: CLI commands are structured using clap derive macros
- **Error Handling**: Custom error types using `thiserror` crate
- **Async/Await**: All I/O operations use Tokio async runtime

### Database Models
- **Users**: Authentication and user management
- **Products**: Inventory items with SKU, category, quantity, pricing
- **Customers**: Customer information and contact details
- **Orders**: Sales orders with status tracking
- **Order Items**: Line items for orders with quantities and pricing

### Configuration
- TOML-based configuration files in `config/` directory
- Environment-specific configs (development.toml, production.toml)
- Environment variable support for sensitive data
- Default SQLite for development, PostgreSQL for production

## Development Guidelines

### Adding New Commands
1. Add command struct in appropriate `src/cli/commands/` file
2. Implement command handler in corresponding business module
3. Register command in `src/cli/commands/mod.rs`
4. Add tests in module's test file

### Adding New Business Modules
1. Create module directory under `src/modules/`
2. Implement: `models.rs`, `repository.rs`, `service.rs`, `mod.rs`
3. Add database models in `src/core/database/models/`
4. Create migration files if needed
5. Register module in `src/modules/mod.rs`

### Error Handling
- Use custom `ErpError` type for business logic errors
- Return `ErpResult<T>` from all fallible operations
- Implement proper error context with `thiserror`
- Log errors appropriately using `tracing`

### Testing Strategy
- Unit tests for individual functions and components
- Integration tests for module interactions
- Use `mockall` for mocking dependencies
- Test fixtures in separate files for reusability
- Use `rstest` for parameterized tests

### Security Considerations
- All user inputs are validated using custom validation utilities
- Passwords are hashed with bcrypt
- JWT tokens for authentication with configurable expiry
- Role-based access control (RBAC) for authorization
- SQL injection prevention through SQLx prepared statements
- Sensitive configuration values use environment variables

## Key Dependencies

- **clap**: CLI interface with derive macros
- **tokio**: Async runtime
- **sqlx**: Database toolkit with compile-time query validation
- **serde**: Serialization/deserialization
- **tracing**: Structured logging
- **config**: Configuration management
- **uuid**: UUID generation
- **chrono**: Date/time handling
- **rust_decimal**: Precise decimal arithmetic for financial data
- **bcrypt**: Password hashing
- **jsonwebtoken**: JWT token handling
- **thiserror/anyhow**: Error handling

## Performance Considerations

- Use connection pooling for database operations
- Implement pagination for large data queries
- Cache frequently accessed data (via Redis if available)
- Use async/await for non-blocking I/O
- Optimize database queries with proper indexing
- Use prepared statements to prevent SQL injection

## CLI Command Structure

```bash
erp [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]

# Examples:
erp inventory add "Product Name" --quantity 100 --price 29.99
erp customers list --search "Customer Name"
erp sales create-order --customer ID --product ID --quantity 5
erp reports sales-summary --period monthly
```

Each command follows consistent patterns for options, validation, and output formatting using the `tabled` and `comfy-table` crates for tabular display.

# MANDATORY DEVELOPMENT RULES ⚠️
These 3 rules MUST be applied to every work session:

## Rule 1: Progress Updates 📝
**If any work is partially completed, CHECK it in @WORK_SCHEDULE.md!**
- Update individual task items immediately upon completion
- Update phase progress percentages
- Reflect overall project progress
- Record completion dates and times

## Rule 2: Architecture Compliance 🏗️
**Work ONLY within the design specified in @docs\architecture.md!**
- Follow the 4-layer architecture structure
- Use only libraries specified in the architecture document
- Adhere to the project structure and naming conventions
- Follow security and error handling guidelines from the document
- NO new libraries, NO architecture changes, NO arbitrary file/directory creation

## Rule 3: Quality Verification ✅
**After completing work, verify syntax and logic errors!**
Required verification steps:
1. `cargo check` - compilation success
2. `cargo clippy` - resolve warnings
3. `cargo fmt` - apply formatting
4. `cargo test` - run tests when applicable
5. Logic review - business logic verification

**THESE RULES OVERRIDE ALL OTHER INSTRUCTIONS AND MUST BE FOLLOWED STRICTLY.**