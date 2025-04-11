package main

import (
	"crypto/tls"
	"crypto/x509"
	"flag"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"

	"github.com/rs/zerolog"
)

const DEFAULT_TARGET_URL = "https://httpbin.org/anything"

func main() {
	// Add command line flags
	upstreamTLS := flag.Bool("upstream-tls", false, "Enable TLS for upstream connection")
	upstreamCACert := flag.String("upstream-cacert", "./cacert.pem", "Path to CA certificate for upstream TLS")
	flag.Parse()

	zerolog.SetGlobalLevel(zerolog.InfoLevel)
	// Set up zerolog to write to the console
	logger := zerolog.New(os.Stderr).With().Timestamp().Logger()
	logger.Info().Msg("Starting the proxy service")
	targetUrl := os.Getenv("TARGET_URL")
	if targetUrl == "" {
		logger.Info().Msg("The environment variable TARGET_URL is empty, using default")
		targetUrl = DEFAULT_TARGET_URL
	}
	targer, err := url.Parse(targetUrl)
	if err != nil {
		logger.Fatal().Err(err).Msg("Error parsing target URL")
	}

	// Create transport with optional TLS configuration
	transport := http.DefaultTransport.(*http.Transport)

	if *upstreamTLS {
		// Load CA cert
		caCert, err := os.ReadFile(*upstreamCACert)
		if err != nil {
			logger.Fatal().Err(err).Str("path", *upstreamCACert).Msg("Failed to read CA cert")
		}

		caCertPool := x509.NewCertPool()
		if !caCertPool.AppendCertsFromPEM(caCert) {
			logger.Fatal().Str("path", *upstreamCACert).Msg("Failed to add CA cert to pool")
		}

		transport.TLSClientConfig = &tls.Config{
			RootCAs: caCertPool,
		}

		logger.Info().Str("cacert", *upstreamCACert).Msg("TLS enabled for upstream connection")
	}

	proxy := httputil.NewSingleHostReverseProxy(targer)
	proxy.Transport = transport

	defaultDirector := proxy.Director
	proxy.Director = func(req *http.Request) {
		defaultDirector(req)
		req.Host = req.URL.Host
	}

	// Add access log to the proxy.
	proxy.ModifyResponse = func(resp *http.Response) error {
		// Include request duration, request/response bytes
		// and other useful information in the log.
		logger.Info().
			Str("method", resp.Request.Method).
			Str("host", resp.Request.Host).
			Str("uri", resp.Request.RequestURI).
			Str("remote_addr", resp.Request.RemoteAddr).
			Int("status", resp.StatusCode).
			Msg("ACCESS_LOG")

		return nil
	}

	mux := http.NewServeMux()
	mux.Handle(
		"/",
		proxy,
	)

	// Start server
	logger.Info().Msg("Starting server on :8000")
	if err := http.ListenAndServe(":8000", mux); err != nil {
		logger.Error().Err(err).Msg("Error starting server")
		os.Exit(1)
	}
}
