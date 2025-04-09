### Run Jmeter
```md
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
    -l "results.jtl"
```

### Netty Backend

```sh
docker run --rm --name netty -p 8688:8688 renukafernando/netty-http-echo-service:0.4.6
```
