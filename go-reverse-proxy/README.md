## HTTPBIN REVERSE PROXY

Configure the target URL (default to https://httpbin.org/anything) via the environment variable `TARGET_URL`.

Invoke the proxy with the following command:

```bash
curl http://localhost:8000/pets
```

### Docker

```shell
docker build -t renukafernando/httpbin-proxy:v1 .
```

### Run

```shell
docker run --rm --name http-proxy -p 8000:8000 \
    --memory="50m" --cpus="0.1" \
    -e TARGET_URL=http://host.docker.internal:8688/test \
    renukafernando/httpbin-proxy:v1
```

### Run with TLS

```shell
docker run --rm --name https-proxy -p 8000:8000 \
    --memory="50m" --cpus="0.1" \
    -v ./cacert.pem:/etc/ssl/certs/netty-cert.pem \
    -e TARGET_URL=https://host.docker.internal:8688/test \
    renukafernando/httpbin-proxy:v1 \
    -upstream-tls \
    -upstream-cacert /etc/ssl/certs/netty-cert.pem
```