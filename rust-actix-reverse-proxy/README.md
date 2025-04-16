# Rust Actix Reverse Proxy

A lightweight, high-performance reverse proxy built with Rust and Actix Web. This proxy supports TLS upstreams, custom CA certificates, and provides detailed access logs.

## Features

- TLS upstream support with custom CA certificates
- Detailed access logs with timestamp, HTTP method, path, host, remote address, status code, and response time
- Configuration via config file or environment variables
- Optimized for low memory and CPU usage
- Docker support with Alpine-based image

## Configuration

The proxy can be configured using either a config file or environment variables.

### Config File (config/default.toml)

```toml
[server]
host = "127.0.0.1"
port = 8080

[upstream]
url = "https://example.com"
# ca_cert_path = ""

[logging]
level = "info"
```

### Environment Variables

All configuration options can be overridden using environment variables with the `APP_` prefix:

- `APP_SERVER_HOST`
- `APP_SERVER_PORT`
- `APP_UPSTREAM_URL`
- `APP_UPSTREAM_CA_CERT_PATH`
- `APP_LOGGING_LEVEL`

## Building

### Local Build

```bash
cargo build --release
```

### Docker Build

```bash
docker build -t renukafernando/rust-actix-reverse-proxy:v1 .
```

## Running

### Local Run

```bash
cargo run --release
```

### Docker Run

```bash
docker run -p 8080:8080 \
  -e APP_UPSTREAM_URL=https://example.com \
  -e APP_UPSTREAM_CA_CERT_PATH=/path/to/ca.crt \
  renukafernando/rust-actix-reverse-proxy:v1
```

## Access Logs

The proxy logs each request in the following format:

```
TIMESTAMP METHOD PATH HOST REMOTE_ADDR STATUS_CODE RESPONSE_TIME_MS
```

Example:
```
2024-03-14 10:15:30.123 GET /api/users example.com 192.168.1.1 200 45ms
```

## Performance Considerations

- The proxy is built with performance in mind, using async I/O and efficient memory management
- TLS connections are reused when possible
- The Alpine-based Docker image minimizes the container size and resource usage
- The proxy runs as a non-root user in the container for security
