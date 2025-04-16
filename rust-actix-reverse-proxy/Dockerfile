# Build stage
FROM rust:1.84-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN adduser -D -H -h /app appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/rust-actix-reverse-proxy /app/
COPY --from=builder /usr/src/app/config.toml /app/config.toml

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the port
EXPOSE 9090

# Run the application
ENTRYPOINT ["./rust-actix-reverse-proxy"]
