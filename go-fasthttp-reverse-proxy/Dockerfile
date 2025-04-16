FROM golang:1.24-alpine AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o proxy .

FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/proxy .
COPY --from=builder /app/config.yaml /etc/proxy/config.yaml

RUN apk --no-cache add ca-certificates tzdata

EXPOSE 8080

ENTRYPOINT ["/app/proxy"] 
