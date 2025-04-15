### Rust Reverse Proxy

### Rust Version

```shell
$ rustc --version
rustc 1.84.0 (9fc6b4312 2025-01-07)
```

#### Local Run

```shell
cargo run
```

```shell
curl http://localhost:3000/hello -v \
    -H 'Content-Type:text/plain' \
    -d 'hello world!' 
```

### Configurations

```shell
UPSTREAM_URL=https://httpbin.org/anything
CERT_PATH=/etc/ssl/certs/netty-cert.pem

```

### Build Docker Image

```shell
docker build -t renukafernando/rust-reverse-proxy:v1 .
```

```shell
docker run --rm --name rust-proxy -p 3000:3000 \
    --memory="10m" --cpus="0.02" \
    -v ./certs/cacert.pem:/etc/ssl/certs/netty-cert.pem \
    -e UPSTREAM_URL=https://httpbin.org/anything \
    renukafernando/rust-reverse-proxy:v1
```

```shell
curl http://localhost:3000/hello -v \
    -H 'Content-Type:text/plain' \
    -d 'hello world!' 
```

### Clean

```shell
docker rm -f rust-proxy
```