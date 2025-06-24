# Multi-stage build for rmz
FROM rust:1.78-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build the application (without --locked to allow dependency resolution)
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    bash \
    coreutils

# Create non-root user
RUN adduser -D -s /bin/bash rmzuser

# Copy the binary from builder
COPY --from=builder /app/target/release/rmz /usr/local/bin/rmz

# Create test directory structure and files as root
RUN mkdir -p /home/rmzuser/test/documents && \
    mkdir -p /home/rmzuser/test/projects && \
    mkdir -p /home/rmzuser/test/temp && \
    mkdir -p /home/rmzuser/test/nested/deep && \
    echo "Important document" > /home/rmzuser/test/documents/important.txt && \
    echo "Project file" > /home/rmzuser/test/projects/main.rs && \
    echo "Temporary data" > /home/rmzuser/test/temp/cache.tmp && \
    echo "Nested file" > /home/rmzuser/test/nested/deep/file.txt && \
    chown -R rmzuser:rmzuser /home/rmzuser

# Switch to non-root user
USER rmzuser
WORKDIR /home/rmzuser

# Set environment variables
ENV RUST_LOG=info
ENV HOME=/home/rmzuser

# Entry point - use sh instead of bash for Alpine
ENTRYPOINT ["/bin/sh"]