#!/bin/bash
set -e

# ERP CLI Development Environment Setup Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

info() {
    echo -e "${BLUE}[SETUP]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check Rust installation
check_rust() {
    if ! command_exists rustc; then
        error "Rust is not installed. Please install Rust from https://rustup.rs/"
    fi

    local rust_version=$(rustc --version | cut -d' ' -f2)
    log "Found Rust version: $rust_version"

    # Check minimum version (1.70)
    local min_version="1.70.0"
    if ! printf '%s\n%s\n' "$min_version" "$rust_version" | sort -V -C; then
        warn "Rust version $rust_version is older than recommended $min_version"
    fi
}

# Install development dependencies
install_dev_dependencies() {
    info "Installing development dependencies..."

    # Install cargo tools
    local tools=(
        "cargo-watch"
        "cargo-edit"
        "cargo-expand"
        "cargo-audit"
        "cargo-tarpaulin"
        "cargo-nextest"
    )

    for tool in "${tools[@]}"; do
        if ! command_exists "$tool"; then
            log "Installing $tool..."
            cargo install "$tool"
        else
            log "$tool is already installed"
        fi
    done
}

# Setup database
setup_database() {
    info "Setting up development database..."

    # Check if PostgreSQL is available
    if command_exists psql; then
        log "PostgreSQL found, setting up development database"

        # Create database if it doesn't exist
        if ! psql -lqt | cut -d \| -f 1 | grep -qw erp_dev; then
            createdb erp_dev || warn "Could not create database erp_dev (may already exist)"
        fi

        # Set database URL
        export DATABASE_URL="postgresql://$(whoami)@localhost/erp_dev"
        echo "DATABASE_URL=$DATABASE_URL" >> .env
        log "Database URL set to: $DATABASE_URL"
    else
        log "PostgreSQL not found, using SQLite for development"
        export DATABASE_URL="sqlite://./dev.db"
        echo "DATABASE_URL=$DATABASE_URL" >> .env
    fi
}

# Setup environment file
setup_env() {
    info "Setting up environment configuration..."

    if [[ ! -f .env ]]; then
        cat > .env << EOF
# Development Environment Configuration
DATABASE_URL=sqlite://./dev.db
REDIS_URL=redis://localhost:6379
JWT_SECRET=dev-jwt-secret-not-for-production
RUST_LOG=debug
ERP_ENV=development
LOG_LEVEL=debug
EOF
        log "Created .env file with development defaults"
    else
        log ".env file already exists"
    fi
}

# Setup git hooks
setup_git_hooks() {
    info "Setting up git hooks..."

    if [[ -d .git ]]; then
        # Pre-commit hook
        cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Check formatting
if ! cargo fmt -- --check; then
    echo "Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
if ! cargo clippy -- -D warnings; then
    echo "Clippy found issues. Please fix them."
    exit 1
fi

# Run tests
if ! cargo test; then
    echo "Tests failed. Please fix them."
    exit 1
fi

echo "All pre-commit checks passed!"
EOF

        chmod +x .git/hooks/pre-commit
        log "Pre-commit hook installed"
    else
        warn "Not a git repository, skipping git hooks setup"
    fi
}

# Create development directories
create_directories() {
    info "Creating development directories..."

    local dirs=(
        "logs"
        "data"
        "temp"
        "scripts"
        "docs"
    )

    for dir in "${dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            log "Created directory: $dir"
        fi
    done
}

# Build the project
build_project() {
    info "Building the project..."

    # Build in debug mode
    if cargo build; then
        log "Debug build successful"
    else
        error "Debug build failed"
    fi

    # Run initial database setup
    log "Running initial database setup..."
    if ./target/debug/erp migrate init; then
        log "Database initialized successfully"
    else
        warn "Database initialization failed (may already be initialized)"
    fi
}

# Run tests
run_tests() {
    info "Running tests..."

    if cargo test; then
        log "All tests passed"
    else
        error "Some tests failed"
    fi
}

# Setup IDE configuration
setup_ide() {
    info "Setting up IDE configuration..."

    # VSCode settings
    if [[ ! -d .vscode ]]; then
        mkdir -p .vscode

        cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },
    "files.exclude": {
        "**/target": true,
        "**/*.db": true
    }
}
EOF

        cat > .vscode/extensions.json << 'EOF'
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb"
    ]
}
EOF

        log "VSCode configuration created"
    fi
}

# Print summary
print_summary() {
    info "Development environment setup complete!"
    echo ""
    log "🎉 Ready to start developing!"
    echo ""
    log "Quick start commands:"
    log "  cargo run -- --help                 # Run the CLI"
    log "  cargo test                          # Run tests"
    log "  cargo watch -x run                  # Run with hot reload"
    log "  cargo clippy                        # Check for lints"
    log "  cargo fmt                           # Format code"
    echo ""
    log "Database commands:"
    log "  cargo run -- migrate status         # Check migration status"
    log "  cargo run -- migrate up             # Run migrations"
    log "  cargo run -- inventory add 'Test' --sku TEST001 --quantity 10 --price 9.99"
    echo ""
    log "Environment:"
    log "  Database: ${DATABASE_URL:-Not set}"
    log "  Log level: ${RUST_LOG:-info}"
    echo ""
    warn "Don't forget to:"
    warn "  1. Review the .env file and adjust settings as needed"
    warn "  2. Install PostgreSQL for production-like development (optional)"
    warn "  3. Install Redis for caching features (optional)"
}

# Main setup process
main() {
    info "Setting up ERP CLI development environment..."
    echo ""

    check_rust
    install_dev_dependencies
    create_directories
    setup_env
    setup_database
    setup_git_hooks
    setup_ide
    build_project
    run_tests
    print_summary
}

# Handle script arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --help)
            echo "ERP CLI Development Environment Setup"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-build    Skip project build"
            echo "  --skip-tests    Skip running tests"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Run main function
main