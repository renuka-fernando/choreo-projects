### Run Jmeter

#### Docker

```shell
colima start --cpu=3 --memory=4
```

#### Netty HTTPs

```shell
docker run --rm --name netty \
    -v ./keystore.p12:/keys/keystore.p12 \
    -p 8688:8688 renukafernando/netty-http-echo-service:0.4.6-arm \
    -m 2g -- --ssl --key-store-file /keys/keystore.p12 --key-store-password '1234'
```

#### Go Reverse Proxy

```shell
docker run --rm --name https-proxy -p 8000:8000 \
    --memory="10m" --cpus="0.01" \
    -v ./cacert.pem:/etc/ssl/certs/netty-cert.pem \
    -e TARGET_URL=https://host.docker.internal:8688/test \
    renukafernando/httpbin-proxy:v1 \
    -upstream-tls \
    -upstream-cacert /etc/ssl/certs/netty-cert.pem
```

#### Run JMeter

```shell
docker stats
```

```shell
jmeter -n -t "perf.jmx" \
    -j "jmeter.log" \
    -Jusers=1 \
    -Jduration="660" \
    -Jhost="localhost" \
    -JhostHeader=localhost \
    -Jport=8000 \
    -Jpath=/echo/1.0.0/ \
    -Jpayload="1KB.json" \
    -Jresponse_size="1024B" \
    -Jprotocol=http \
    -Jtokens="${HOME}/jwt-tokens-${user_count}.csv" \
    -Jrpm=300000 \
    -l "results.jtl"
```

#### Get Summary

```shell
tar -czf results.tar.gz results.jtl
java -jar jtl-splitter-0.4.6-SNAPSHOT.jar -p -s -u MINUTES -t 1 -f "results.jtl"
cat results-measurement-summary.json
rm results.jtl results-measurement.jtl results-warmup.jtl results-warmup-summary.json results-measurement-summary.json
```
