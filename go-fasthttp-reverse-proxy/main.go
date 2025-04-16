package main

import (
	"crypto/tls"
	"crypto/x509"
	"net/url"
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/spf13/viper"
	"github.com/valyala/fasthttp"
)

type Config struct {
	Server struct {
		ListenAddr    string `mapstructure:"listen_addr"`
		ReadTimeout   string `mapstructure:"read_timeout"`
		WriteTimeout  string `mapstructure:"write_timeout"`
		IdleTimeout   string `mapstructure:"idle_timeout"`
		MaxHeaderSize int    `mapstructure:"max_header_size"`
		MaxBodySize   int    `mapstructure:"max_body_size"`
	} `mapstructure:"server"`
	Upstream struct {
		URL                 string `mapstructure:"url"`
		CACertPath          string `mapstructure:"ca_cert_path"`
		InsecureSkipVerify  bool   `mapstructure:"insecure_skip_verify"`
		MaxConnsPerHost     int    `mapstructure:"max_conns_per_host"`
		MaxIdleConns        int    `mapstructure:"max_idle_conns"`
		MaxIdleConnDuration string `mapstructure:"max_idle_conn_duration"`
		MaxConnDuration     string `mapstructure:"max_conn_duration"`
		MaxConnWaitTimeout  string `mapstructure:"max_conn_wait_timeout"`
	} `mapstructure:"upstream"`
}

func loadConfig() (*Config, error) {
	viper.SetConfigName("config")
	viper.SetConfigType("yaml")
	viper.AddConfigPath(".")
	viper.AddConfigPath("/etc/proxy/")

	viper.SetEnvPrefix("PROXY")
	viper.AutomaticEnv()

	if err := viper.ReadInConfig(); err != nil {
		return nil, err
	}

	var config Config
	if err := viper.Unmarshal(&config); err != nil {
		return nil, err
	}

	return &config, nil
}

func setupLogger() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix
	log.Logger = log.Output(zerolog.ConsoleWriter{
		Out:        os.Stdout,
		TimeFormat: time.RFC3339,
		NoColor:    true,
	})
}

func createTLSConfig(config *Config) (*tls.Config, error) {
	tlsConfig := &tls.Config{
		InsecureSkipVerify: config.Upstream.InsecureSkipVerify,
	}

	// Always use system's default CA certificates
	systemPool, err := x509.SystemCertPool()
	if err != nil {
		return nil, err
	}
	tlsConfig.RootCAs = systemPool

	// If custom CA cert is provided, append it to the system pool
	if config.Upstream.CACertPath != "" {
		caCert, err := os.ReadFile(config.Upstream.CACertPath)
		if err != nil {
			return nil, err
		}

		if !systemPool.AppendCertsFromPEM(caCert) {
			return nil, err
		}
	}

	return tlsConfig, nil
}

func main() {
	setupLogger()

	config, err := loadConfig()
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to load configuration")
	}

	tlsConfig, err := createTLSConfig(config)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to create TLS config")
	}

	upstramUrl, err := url.Parse(config.Upstream.URL)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to parse upstream URL")
	}
	upstreamHost := upstramUrl.Host

	client := &fasthttp.Client{
		TLSConfig:       tlsConfig,
		MaxConnsPerHost: config.Upstream.MaxConnsPerHost,
		MaxIdleConnDuration: func() time.Duration {
			d, _ := time.ParseDuration(config.Upstream.MaxIdleConnDuration)
			return d
		}(),
		MaxConnDuration: func() time.Duration {
			d, _ := time.ParseDuration(config.Upstream.MaxConnDuration)
			return d
		}(),
		MaxConnWaitTimeout: func() time.Duration {
			d, _ := time.ParseDuration(config.Upstream.MaxConnWaitTimeout)
			return d
		}(),
	}

	server := &fasthttp.Server{
		Handler: func(ctx *fasthttp.RequestCtx) {
			start := time.Now()

			req := &fasthttp.Request{}
			ctx.Request.CopyTo(req)
			req.SetRequestURI(config.Upstream.URL + string(ctx.Path()))
			req.SetHost(upstreamHost)

			resp := &fasthttp.Response{}
			if err := client.Do(req, resp); err != nil {
				log.Error().Err(err).Msg("Failed to proxy request")
				ctx.SetStatusCode(fasthttp.StatusBadGateway)
				return
			}

			// Remove Server header
			resp.Header.Del("Server")

			resp.CopyTo(&ctx.Response)

			duration := time.Since(start)
			log.Info().
				Str("method", string(ctx.Method())).
				Str("path", string(ctx.Path())).
				Str("host", string(req.Host())).
				Str("remote_addr", ctx.RemoteAddr().String()).
				Int("status_code", resp.StatusCode()).
				Dur("response_time", duration).
				Msg("Access log")
		},
		ReadTimeout: func() time.Duration {
			d, _ := time.ParseDuration(config.Server.ReadTimeout)
			return d
		}(),
		WriteTimeout: func() time.Duration {
			d, _ := time.ParseDuration(config.Server.WriteTimeout)
			return d
		}(),
		IdleTimeout: func() time.Duration {
			d, _ := time.ParseDuration(config.Server.IdleTimeout)
			return d
		}(),
		MaxRequestBodySize: config.Server.MaxBodySize,
	}

	log.Info().Str("addr", config.Server.ListenAddr).Msg("Starting server")
	if err := server.ListenAndServe(config.Server.ListenAddr); err != nil {
		log.Fatal().Err(err).Msg("Failed to start server")
	}
}
