FROM golang:1.24.2 as builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 go build -o httpbin-proxy

FROM alpine:3.21.3
COPY --from=builder /app/httpbin-proxy /usr/local/bin/httpbin-proxy
ENTRYPOINT ["httpbin-proxy"]
