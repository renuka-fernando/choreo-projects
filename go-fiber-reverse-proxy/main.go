package main

import (
	"crypto/tls"
	"crypto/x509"
	"io/ioutil"
	"os"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/proxy"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/valyala/fasthttp"
)

type Config struct {
	ListenAddr     string
	UpstreamURL    string
	CACertPath     string
	InsecureSkip   bool
	ReadTimeout    time.Duration
	WriteTimeout   time.Duration
	MaxConnections int
}

func main() {
	// Configure zerolog
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339})

	// Default configuration
	config := Config{
		ListenAddr:     ":8080",
		UpstreamURL:    "https://httpbin.org/anything",
		CACertPath:     "",
		InsecureSkip:   false,
		ReadTimeout:    10 * time.Second,
		WriteTimeout:   10 * time.Second,
		MaxConnections: 1000,
	}

	// Create Fiber app with optimized settings
	app := fiber.New(fiber.Config{
		ReadTimeout:  config.ReadTimeout,
		WriteTimeout: config.WriteTimeout,
		IdleTimeout:  120 * time.Second,
		// Optimize memory usage
		EnableTrustedProxyCheck: false,
		DisableStartupMessage:   true,
	})

	// Custom middleware for access logging
	app.Use(func(c *fiber.Ctx) error {
		start := time.Now()
		err := c.Next()
		latency := time.Since(start)

		log.Info().
			Str("timestamp", time.Now().Format(time.RFC3339)).
			Str("method", c.Method()).
			Str("path", c.Path()).
			Str("host", c.Hostname()).
			Str("remote_addr", c.IP()).
			Int("status", c.Response().StatusCode()).
			Dur("latency", latency).
			Msg("access log")

		return err
	})

	// Create HTTP client with TLS configuration
	client := &fasthttp.Client{
		ReadTimeout:     config.ReadTimeout,
		WriteTimeout:    config.WriteTimeout,
		MaxConnsPerHost: config.MaxConnections,
	}

	// Configure TLS if CA cert is provided
	if config.CACertPath != "" {
		caCert, err := ioutil.ReadFile(config.CACertPath)
		if err != nil {
			log.Fatal().Err(err).Msg("failed to read CA cert")
		}

		caCertPool := x509.NewCertPool()
		caCertPool.AppendCertsFromPEM(caCert)

		client.TLSConfig = &tls.Config{
			RootCAs:            caCertPool,
			InsecureSkipVerify: config.InsecureSkip,
		}
	}

	// Setup proxy route
	app.All("/*", func(c *fiber.Ctx) error {
		return proxy.Do(c, config.UpstreamURL, client)
	})

	// Start server
	log.Info().Str("addr", config.ListenAddr).Msg("starting reverse proxy server")
	if err := app.Listen(config.ListenAddr); err != nil {
		log.Fatal().Err(err).Msg("failed to start server")
	}
}
