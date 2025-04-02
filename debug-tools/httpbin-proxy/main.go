package main

import (
	"log/slog"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
)

const DEFAULT_TARGET_URL = "https://httpbin.org/anything"

func main() {
	slog.Info("Starting the proxy service")
	targetUrl := os.Getenv("TARGET_URL")
	if targetUrl == "" {
		slog.Info("The environment variable TARGET_URL is empty, using default")
		targetUrl = DEFAULT_TARGET_URL
	}
	targer, err := url.Parse(targetUrl)
	if err != nil {
		slog.Error("Error parsing target URL", err)
		os.Exit(1)
	}
	proxy := httputil.NewSingleHostReverseProxy(targer)

	// Add access log to the proxy.
	proxy.ModifyResponse = func(resp *http.Response) error {
		// Include request duration, request/response bytes
		// and other useful information in the log.
		slog.Info("Request",
			"method", resp.Request.Method,
			"host", resp.Request.Host,
			"uri", resp.Request.RequestURI,
			"remote_addr", resp.Request.RemoteAddr,
			"status", resp.StatusCode,
		)

		return nil
	}

	mux := http.NewServeMux()
	mux.Handle(
		"/",
		proxy,
	)

	// Start server
	slog.Info("Starting server on :8000")
	if err := http.ListenAndServe(":8000", mux); err != nil {
		slog.Error("error starting server", "error", err)
		os.Exit(1)
	}
}
