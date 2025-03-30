# Build stage
FROM rust:1.70-slim-bullseye as builder

WORKDIR /usr/src/app

# Copy manifests first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/ranx .

# Copy configuration
COPY config.yaml .

# Create non-root user
RUN useradd -m -u 1000 ranx && \
    chown -R ranx:ranx /app

USER ranx

# Expose the port the proxy will listen on
EXPOSE 8080

# Run the proxy
CMD ["./ranx"] 
