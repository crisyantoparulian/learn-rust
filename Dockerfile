# Use the official Rust image as the builder stage
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy Cargo.toml and Cargo.lock first to leverage Docker caching
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies (this will be cached as long as Cargo.toml and Cargo.lock don’t change)
RUN cargo fetch

# Copy the rest of the source code
COPY . .

# Build the Rust application in release mode
RUN cargo build --release

# Use a minimal runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/learn_rust /usr/local/bin/learn_rust

# Set environment variable (optional)
ENV RUST_LOG=info

# Define the entrypoint for the container
CMD ["learn_rust"]