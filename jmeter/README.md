### Run Jmeter

```shell
jmeter -n -t "perf.jmx" \
    -j "jmeter.log" \
    -Jusers=1 \
    -Jduration="300" \
    -Jhost="localhost" \
    -JhostHeader=localhost \
    -Jport=8000 \
    -Jpath=/echo/1.0.0/ \
    -Jpayload="1KB.json" \
    -Jresponse_size="1024B" \
    -Jprotocol=http \
    -Jtokens="${HOME}/jwt-tokens-${user_count}.csv" \
    -Jrpm=300 \
    -l "results.jtl"
```

### Netty Backend

```shell
docker run --rm --name netty \
    -v ./keystore.p12:/keys/keystore.p12 \
    -p 8688:8688 renukafernando/netty-http-echo-service:0.4.6-arm \
    -m 2g -- --ssl --key-store-file /keys/keystore.p12 --key-store-password '1234'
```
