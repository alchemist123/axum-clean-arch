# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Install required system dependencies for sqlx
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
# Docker will cache this layer if Cargo.toml and Cargo.lock don't change
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/profile-blog-axum /app/profile-blog-axum

# Copy migrations
COPY migrations ./migrations

# Create logs directory
RUN mkdir -p logs

# Expose port
EXPOSE 3000

# Set environment variables (can be overridden)
ENV RUST_LOG=info

# Run the application
CMD ["./profile-blog-axum"]

