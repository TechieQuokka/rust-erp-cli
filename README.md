# Rust ERP CLI System

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, modular Enterprise Resource Planning (ERP) Command Line Interface system built with Rust. This system provides comprehensive business management capabilities through an intuitive CLI interface.

## 🚀 Features

- **📦 Inventory Management**: Complete product lifecycle management
- **👥 Customer Management**: Customer information and relationship tracking
- **💰 Sales Management**: Order processing and invoice generation
- **📊 Reporting System**: Comprehensive business analytics and reports
- **🔐 Authentication**: JWT-based user authentication with RBAC
- **⚡ High Performance**: Built with Rust for maximum performance and safety
- **🛡️ Security**: Advanced security features with data protection
- **📈 Scalable**: Modular architecture supporting easy extensions

## 🏗️ Architecture

The system follows a 4-layer modular architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                     ERP CLI System                         │
├─────────────────────────────────────────────────────────────┤
│  CLI Interface Layer                                       │
│  ┌─────────────────┬─────────────────┬─────────────────┐   │
│  │   Commands      │    Parser       │    Validator    │   │
│  └─────────────────┴─────────────────┴─────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Business Logic Layer                                      │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐ │
│  │Inventory │  Sales   │Customers │ Reports  │  Config  │ │
│  │ Module   │  Module  │  Module  │  Module  │  Module  │ │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Core Services Layer                                       │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ Auth Service│Database Svc │Config Svc  │  Log Service│ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Data Layer                                                │
│  ┌─────────────────┬─────────────────┬─────────────────┐   │
│  │   PostgreSQL    │     SQLite      │     Redis       │   │
│  │   (Production)  │   (Development) │   (Caching)     │   │
│  └─────────────────┴─────────────────┴─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 🛠️ Quick Start

### Prerequisites

- **Rust** (1.75 or later) - [Install Rust](https://rustup.rs/)
- **PostgreSQL** (v14+) for production
- **SQLite** (v3.35+) for development
- **Redis** (v6+) for caching (optional)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/your-org/erp-cli.git
cd erp-cli
```

2. Build the project:
```bash
cargo build --release
```

3. Initialize the database:
```bash
./target/release/erp setup --init-db
```

4. Create your first user:
```bash
./target/release/erp auth register --username admin --email admin@company.com
```

### Basic Usage

```bash
# Show help
erp --help

# Inventory management
erp inventory add "Product Name" --quantity 100 --price 29.99 --category Electronics
erp inventory list --low-stock
erp inventory update PRODUCT_ID --quantity 150

# Customer management
erp customers add "Customer Name" --email customer@example.com --phone "123-456-7890"
erp customers list --search "Customer Name"

# Sales management
erp sales create-order --customer CUSTOMER_ID --product PRODUCT_ID --quantity 5
erp sales list-orders --status pending
erp sales generate-invoice ORDER_ID

# Reports
erp reports sales-summary --period monthly
erp reports inventory-status
erp reports customer-analysis --top 10
```

## 📖 Documentation

- [Architecture Design](docs/architecture.md) - Complete system architecture
- [User Guide](docs/user-guide.md) - Comprehensive user manual
- [API Reference](docs/api-reference.md) - API documentation
- [Development Guide](docs/development.md) - Developer handbook

## 🧪 Development

### Setting up Development Environment

1. Install development tools:
```bash
# Code quality tools
cargo install cargo-clippy
cargo install cargo-fmt

# Testing tools
cargo install cargo-nextest
cargo install cargo-tarpaulin

# Documentation
cargo install cargo-doc
```

2. Run tests:
```bash
# Run all tests
cargo nextest run

# Run with coverage
cargo tarpaulin --out Html
```

3. Code quality checks:
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Build documentation
cargo doc --open
```

### Project Structure

```
erp-cli/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── cli/                 # CLI interface layer
│   ├── modules/             # Business logic modules
│   ├── core/                # Core services
│   ├── utils/               # Utilities
│   └── api/                 # REST API (optional)
├── migrations/              # Database migrations
├── config/                  # Configuration files
├── docs/                    # Documentation
└── tests/                   # Tests
```

## 🚦 Testing

The project includes comprehensive testing at multiple levels:

- **Unit Tests**: Test individual components
- **Integration Tests**: Test module interactions
- **End-to-End Tests**: Test complete user workflows

```bash
# Run all tests
cargo test

# Run specific test module
cargo test inventory

# Run with detailed output
cargo test -- --nocapture
```

## 🔒 Security

- JWT-based authentication
- Role-based access control (RBAC)
- Password hashing with bcrypt
- SQL injection prevention
- Input validation and sanitization
- Audit logging for all operations

## 📊 Performance

- **Memory**: < 50MB baseline usage
- **Response Time**: < 100ms for most operations
- **Database**: Optimized queries with proper indexing
- **Concurrency**: Async/await with Tokio runtime

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and development process.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization framework

## 📞 Support

- 📧 Email: support@erp-cli.com
- 📖 Documentation: [docs.erp-cli.com](https://docs.erp-cli.com)
- 🐛 Issues: [GitHub Issues](https://github.com/your-org/erp-cli/issues)

---

**Made with ❤️ and Rust**