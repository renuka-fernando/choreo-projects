use hyper::{
    Body, Client, Request, Response, Server, Uri,
    service::{make_service_fn, service_fn},
    server::conn::AddrStream,
};
use std::{convert::Infallible, fs::File, io::BufReader, net::SocketAddr, sync::Arc, time::Instant};
use hyper_rustls::HttpsConnectorBuilder;
use rustls::{Certificate, ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use rustls_native_certs;
use chrono::Local;
use hyper::client::HttpConnector;
use dotenv::dotenv;
use std::env;
use once_cell::sync::Lazy;
use tracing::{info, error};
use tracing_subscriber;

static UPSTREAM_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("UPSTREAM_URL").unwrap_or_else(|_| "https://localhost:8443".to_string())
});

static CERT_PATH: Lazy<Option<String>> = Lazy::new(|| env::var("CERT_PATH").ok());

#[derive(Clone)]
struct AppState {
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>>,
}

fn load_certs() -> Option<Vec<Certificate>> {
    let Some(path) = CERT_PATH.as_ref() else {
        return None;
    };

    let file = File::open(path).unwrap_or_else(|e| {
        panic!("Failed to open cert file at {}: {}", path, e);
    });

    let mut reader = BufReader::new(file);
    let certs = certs(&mut reader).expect("Failed to parse PEM file");

    Some(certs.into_iter().map(Certificate).collect())
}

fn make_https_client(certs: Option<Vec<Certificate>>) -> Client<hyper_rustls::HttpsConnector<HttpConnector>> {
    let mut root_store = RootCertStore::empty();

    if let Some(cert_list) = certs {
        for cert in &cert_list {
            root_store.add(cert).expect("Invalid certificate format");
        }
    } else {
        let native_certs = rustls_native_certs::load_native_certs()
            .expect("Failed to load platform certificates");

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
        .build(https)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let certs = load_certs();
    let client = make_https_client(certs);
    let state = Arc::new(AppState { client });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_service = make_service_fn(move |_conn: &AddrStream| {
        let state = state.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                proxy_handler(req, state.clone())
            }))
        }
    });

    info!(%addr, upstream = %UPSTREAM_URL.as_str(), cert_path = ?CERT_PATH.as_deref().unwrap_or("<system default>"), "Reverse proxy starting");

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
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    let target_uri = format!("{}{}", UPSTREAM_URL.as_str(), uri)
        .parse::<Uri>()
        .unwrap();

    parts.uri = target_uri;
    let new_req = Request::from_parts(parts, body);

    let result = state.client.request(new_req).await;
    let duration = start.elapsed().as_millis();

    match &result {
        Ok(response) => {
            info!(
                timestamp = %now.format("%Y-%m-%d %H:%M:%S"),
                %method,
                path = %path,
                query = %query,
                status = %response.status(),
                elapsed_ms = %duration,
                "Request proxied"
            );
        }
        Err(e) => {
            error!(
                timestamp = %now.format("%Y-%m-%d %H:%M:%S"),
                %method,
                path = %path,
                query = %query,
                error = %e,
                elapsed_ms = %duration,
                "Proxy error"
            );
        }
    }

    result
}
