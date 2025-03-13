FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy Cargo.toml and Cargo.lock separately to leverage Docker's cache
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/lib.rs to allow dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -r src

# Copy the source code and build
COPY . ./
RUN cargo build --release

# Use a minimal runtime environment
FROM debian:bookworm-slim
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/weather_track ./

# Set the binary as the entrypoint
CMD ["./weather_track"]
