# Multi-stage build for hodei-server
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/hodei-server/Cargo.toml ./crates/hodei-server/

# Copy other crate manifests for dependency resolution
RUN mkdir -p crates/hodei-ir crates/hodei-engine crates/hodei-dsl
COPY crates/hodei-ir/Cargo.toml crates/hodei-ir/
COPY crates/hodei-engine/Cargo.toml crates/hodei-engine/
COPY crates/hodei-dsl/Cargo.toml crates/hodei-dsl/

# Build the application
RUN cargo build --release --bin hodei-server

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false hodei

# Copy the binary from builder stage
COPY --from=builder /app/target/release/hodei-server /usr/local/bin/

# Create app directory
WORKDIR /app

# Create necessary directories
RUN mkdir -p /app/logs /app/config && \
    chown -R hodei:hodei /app

# Switch to non-root user
USER hodei

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV HODEI_BIND_ADDRESS=0.0.0.0:8080
ENV RUST_LOG=info

# Run the server
CMD ["hodei-server"]
