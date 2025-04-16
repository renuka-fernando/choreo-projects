# Go FastHTTP Reverse Proxy

A high-performance reverse proxy built with Go using the FastHTTP library and zerolog for logging.

## Features

- High-performance reverse proxy using FastHTTP
- TLS support with custom CA certificates
- Detailed access logging with zerolog
- Configurable through YAML config file or environment variables
- Optimized for low memory and CPU usage
- Docker support with Alpine-based image

## Configuration

The proxy can be configured using either a YAML config file or environment variables. The configuration file should be placed at `/etc/proxy/config.yaml` or in the current directory.

### Configuration Options

```yaml
server:
  listen_addr: ":8080"
  read_timeout: "5s"
  write_timeout: "5s"
  idle_timeout: "60s"
  max_header_size: 8192
  max_body_size: 10485760  # 10MB

upstream:
  url: "https://example.com"
  ca_cert_path: "/etc/proxy/ca.crt"
  insecure_skip_verify: false
  max_conns_per_host: 100
  max_idle_conns: 100
  max_idle_conn_duration: "90s"
  max_conn_duration: "0s"
  max_conn_wait_timeout: "0s"
```

### Environment Variables

All configuration options can be overridden using environment variables with the `PROXY_` prefix. For example:

- `PROXY_SERVER_LISTEN_ADDR`
- `PROXY_UPSTREAM_URL`
- `PROXY_UPSTREAM_CA_CERT_PATH`

## Building

```bash
go build -o proxy
```

## Running

```bash
./proxy
```

## Docker

Build the Docker image:

```bash
docker build -t renukafernando/go-fasthttp-reverse-proxy:v1 .
```

Run the container:

```bash
docker run --rm --name go-fasthttp-proxy -p 8080:8080 \
  -v ./config.yaml:/etc/proxy/config.yaml \
  -v ./cacert.pem:/app/cacert.pem \
  renukafernando/go-fasthttp-reverse-proxy:v1
```

## Access Logs

The proxy logs detailed access information for each request, including:
- Timestamp
- HTTP Method
- Path
- Host
- Remote Address
- Status Code
- Response Time

Example log output:
```
2024-03-14T10:15:30Z INF method=GET path=/api/v1/users host=example.com remote_addr=192.168.1.1 status_code=200 response_time=0.023s
``` 
