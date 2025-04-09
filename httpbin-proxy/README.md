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
docker run --rm -p --name http-proxy 8000:8000 \
    -e TARGET_URL=http://localhost:8688/test \
    renukafernando/httpbin-proxy:v1
```