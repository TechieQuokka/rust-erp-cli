# Multi-stage build for optimized production image
FROM rust:1.70-slim as builder

# Install necessary system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY config ./config

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 erp

# Create necessary directories
RUN mkdir -p /app/config /app/logs /app/data && \
    chown -R erp:erp /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/erp /usr/local/bin/erp
COPY --from=builder /app/config /app/config

# Set ownership
RUN chown erp:erp /usr/local/bin/erp

# Switch to non-root user
USER erp
WORKDIR /app

# Default environment variables
ENV RUST_LOG=info
ENV ERP_CONFIG_PATH=/app/config
ENV DATABASE_URL=sqlite:///app/data/erp.db

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD erp migrate test || exit 1

# Expose port (if running as server in future)
EXPOSE 8080

# Default command
CMD ["erp", "--help"]