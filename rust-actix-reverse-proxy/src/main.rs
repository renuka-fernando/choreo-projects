use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    web, App, Error, HttpServer, Responder,
};
use chrono::Utc;
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{info, warn, debug, error};
use reqwest::Client;
use rustls::RootCertStore;
use rustls_native_certs::load_native_certs;
use serde::Deserialize;
use std::time::Instant;
use url::Url;

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    upstream: UpstreamConfig,
    logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct UpstreamConfig {
    url: String,
    ca_cert_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 9090,
            },
            upstream: UpstreamConfig {
                url: "https://httpbin.org/anything".to_string(),
                ca_cert_path: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        }
    }
}

#[derive(Clone)]
struct ProxyClient {
    client: Client,
    upstream_url: String,
    upstream_host: String,
}

impl ProxyClient {
    async fn new(config: &UpstreamConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut root_store = RootCertStore::empty();
        
        // Load system certificates
        match load_native_certs() {
            Ok(certs) => {
                debug!("Loaded {} system certificates", certs.len());
                for cert in certs {
                    if let Err(e) = root_store.add(cert) {
                        warn!("Failed to add system certificate: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to load system certificates: {}", e);
            }
        }

        // Load custom CA certificate if provided
        if let Some(ca_path) = &config.ca_cert_path {
            debug!("Loading custom CA certificate from: {}", ca_path);
            match std::fs::read_to_string(ca_path) {
                Ok(ca_cert_pem) => {
                    let mut cursor = std::io::Cursor::new(ca_cert_pem);
                    let cert_iter = rustls_pemfile::certs(&mut cursor);
                    let mut cert_count = 0;
                    
                    for cert_result in cert_iter {
                        match cert_result {
                            Ok(cert) => {
                                if let Err(e) = root_store.add(rustls::pki_types::CertificateDer::from(cert)) {
                                    warn!("Failed to add custom certificate: {}", e);
                                } else {
                                    cert_count += 1;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse certificate: {}", e);
                            }
                        }
                    }
                    debug!("Successfully loaded {} certificates from custom CA", cert_count);
                }
                Err(e) => {
                    error!("Failed to read CA certificate file: {}", e);
                    return Err(format!("Failed to read CA certificate file: {}", e).into());
                }
            }
        }

        debug!("Total certificates in root store: {}", root_store.len());

        // Create a client with rustls
        let client = Client::builder()
            .use_rustls_tls()
            .danger_accept_invalid_certs(true) // Accept self-signed certificates for development
            .build()?;

        // Extract upstream host from URL
        let upstream_url = config.url.clone();
        let upstream_host = Url::parse(&upstream_url)
            .map(|url| url.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(ProxyClient {
            client,
            upstream_url,
            upstream_host,
        })
    }
}

struct AccessLogger;

impl<S, B> Transform<S, ServiceRequest> for AccessLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AccessLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AccessLoggerMiddleware { service }))
    }
}

struct AccessLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AccessLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();
        let host = req.headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let remote_addr = req.connection_info().peer_addr().unwrap_or("unknown").to_string();

        // Get the upstream host from the app data
        let upstream_host = req.app_data::<web::Data<ProxyClient>>()
            .map(|client| client.upstream_host.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // We can't clone the request, so we'll just use it directly
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed();
            let status = res.status();

            info!(
                "{} {} {} {} {} {} {} {}ms",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                method,
                path,
                host,
                upstream_host,
                remote_addr,
                status.as_u16(),
                duration.as_millis()
            );

            Ok(res)
        })
    }
}

async fn proxy_handler(
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
    client: web::Data<ProxyClient>,
) -> Result<impl Responder, Error> {
    let upstream_url = format!("{}{}", client.upstream_url, req.uri().path());
    debug!("Proxying request to: {}", upstream_url);
    
    let mut upstream_req = client.client.request(req.method().clone(), &upstream_url);

    // Copy headers
    for (name, value) in req.headers() {
        if name != header::HOST {
            upstream_req = upstream_req.header(name, value);
        }
    }

    // Add body if present
    if !body.is_empty() {
        upstream_req = upstream_req.body(body);
    }

    match upstream_req.send().await {
        Ok(response) => {
            let status = response.status();
            debug!("Received response from upstream with status: {}", status);
            
            let mut builder = actix_web::HttpResponse::build(status);

            // Copy headers from upstream response
            for (name, value) in response.headers() {
                builder.append_header((name, value));
            }

            let body = response.bytes().await.map_err(|e| {
                warn!("Error reading response body: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

            Ok(builder.body(body))
        }
        Err(e) => {
            error!("Upstream request failed: {}", e);
            Err(actix_web::error::ErrorBadGateway(e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration with fallback to defaults
    let config = match config::Config::builder()
        .add_source(config::File::with_name("config.toml").required(false))
        .add_source(config::Environment::with_prefix("APP"))
        .build() {
            Ok(config) => config.try_deserialize::<Config>().unwrap_or_default(),
            Err(_) => Config::default(),
        };

    // Initialize logging
    std::env::set_var("RUST_LOG", &config.logging.level);
    env_logger::init();

    // Initialize proxy client
    let proxy_client = ProxyClient::new(&config.upstream)
        .await
        .expect("Failed to initialize proxy client");

    info!(
        "Starting reverse proxy server on {}:{}",
        config.server.host, config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(AccessLogger)
            .app_data(web::Data::new(proxy_client.clone()))
            .default_service(web::to(proxy_handler))
    })
    .bind((config.server.host, config.server.port))?
    .run()
    .await
}
