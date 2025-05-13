# Builder stage
FROM rust:1.82 AS builder

WORKDIR /usr/src/app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.20

WORKDIR /app

# Install necessary runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/rust-app .

# Set the binary as the entrypoint
ENTRYPOINT ["./rust-app"]