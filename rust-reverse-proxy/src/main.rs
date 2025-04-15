use hyper::{
    body::Body,
    client::{Client, HttpConnector},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Request, Response, Server, Uri,
};
use hyper_rustls::HttpsConnectorBuilder;
use rustls::{Certificate, ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use rustls_native_certs;
use std::{
    convert::Infallible,
    fs::File,
    io::{BufReader, stdout},
    net::SocketAddr,
    sync::Arc,
    time::Instant,
};
use chrono::Local;
use dotenv::dotenv;
use once_cell::sync::{Lazy, OnceCell};
use tracing::{error, info};
use tracing_appender::non_blocking;
use tracing_subscriber::{EnvFilter};

static UPSTREAM_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    std::env::var("UPSTREAM_URL").unwrap_or_else(|_| "https://localhost:8443".to_string())
});
static CERT_PATH: Lazy<Option<String>> = Lazy::new(|| {
    std::env::var("CERT_PATH").ok()
});
static LOG_GUARD: OnceCell<tracing_appender::non_blocking::WorkerGuard> = OnceCell::new();

struct AppState {
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>>,
}

fn init_logging() {
    let (non_blocking_writer, guard) = non_blocking(stdout()); // Log to stdout
    LOG_GUARD.set(guard).unwrap(); // Prevent dropped logs

    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_thread_ids(true)
        .with_target(false)
        .with_ansi(false) // Disable colors
        .init();
}

fn make_https_client_with_custom_cert() -> Client<hyper_rustls::HttpsConnector<HttpConnector>> {
    let mut root_store = RootCertStore::empty();

    if let Some(ref cert_path) = *CERT_PATH {
        let file = File::open(cert_path).expect("Failed to open cert file");
        let mut reader = BufReader::new(file);
        let certs = certs(&mut reader).expect("Failed to parse PEM file");
        for cert in certs {
            root_store
                .add(&Certificate(cert))
                .expect("Invalid certificate format");
        }
    } else {
        let native_certs = rustls_native_certs::load_native_certs()
            .expect("Could not load platform certs");
        for cert in native_certs {
            root_store.add(&Certificate(cert.0)).unwrap();
        }
    }

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let https = HttpsConnectorBuilder::new()
        .with_tls_config(config)
        .https_or_http()
        .enable_http1()
        .build();

    Client::builder()
        .pool_max_idle_per_host(64)
        .build::<_, hyper::Body>(https)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logging();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let client = make_https_client_with_custom_cert();
    let state = Arc::new(AppState { client });

    let make_service = make_service_fn(move |_conn: &AddrStream| {
        let state = Arc::clone(&state);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                proxy_handler(req, Arc::clone(&state))
            }))
        }
    });

    info!("Reverse proxy listening on http://{}", addr);
    info!("Upstream: {}", UPSTREAM_URL.as_str());
    if let Some(path) = CERT_PATH.as_ref() {
        info!("Custom cert path: {}", path);
    } else {
        info!("Using OS certificate store");
    }

    let server = Server::bind(&addr)
        .tcp_nodelay(true)
        .serve(make_service);

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}

async fn proxy_handler(
    req: Request<Body>,
    state: Arc<AppState>,
) -> Result<Response<Body>, hyper::Error> {
    let start = Instant::now();
    let now = Local::now();

    let (mut parts, body) = req.into_parts();
    let method = parts.method.clone();
    let uri = parts.uri.clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();

    let target_uri = format!("{}{}", UPSTREAM_URL.as_str(), uri)
        .parse::<Uri>()
        .unwrap();

    parts.uri = target_uri;
    let new_req = Request::from_parts(parts, body);

    let result = state.client.request(new_req).await;
    let duration = start.elapsed().as_millis();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let method = method.to_string();
    let status = result
        .as_ref()
        .map(|res| res.status().as_u16())
        .unwrap_or(0);
    let error = result
        .as_ref()
        .err()
        .map(|e| e.to_string());

    tokio::spawn(async move {
        match error {
            Some(err) => {
                error!(
                    %timestamp,
                    %method,
                    path = %path,
                    query = %query,
                    %err,
                    elapsed_ms = %duration,
                    "Proxy error"
                );
            }
            None => {
                info!(
                    %timestamp,
                    %method,
                    path = %path,
                    query = %query,
                    status = %status,
                    elapsed_ms = %duration,
                    "Request proxied"
                );
            }
        }
    });

    result
}
