FROM rust:1.84-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.19
RUN apk add --no-cache libgcc libstdc++ openssl

WORKDIR /app

COPY --from=builder /app/target/release/rust-reverse-proxy /app/rust-reverse-proxy
EXPOSE 3000

ENTRYPOINT ["/app/rust-reverse-proxy"]
