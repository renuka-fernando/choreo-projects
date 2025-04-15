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
use tokio::sync::RwLock;

// Load config from env or use defaults
static UPSTREAM_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("UPSTREAM_URL").unwrap_or_else(|_| "https://httpbin.org/anything".to_string())
});

static CERT_PATH: Lazy<Option<String>> = Lazy::new(|| {
    env::var("CERT_PATH").ok()
});

// Store certs (if any) globally
static GLOBAL_CERTS: Lazy<Arc<RwLock<Option<Vec<Certificate>>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

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

fn make_https_client_with_custom_cert(certs: Option<&[Certificate]>) -> Client<hyper_rustls::HttpsConnector<HttpConnector>> {
    let mut root_store = RootCertStore::empty();

    if let Some(cert_list) = certs {
        for cert in cert_list {
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

    Client::builder().build(https)
}

#[tokio::main]
async fn main() {
    let certs = load_certs();
    {
        let mut certs_lock = GLOBAL_CERTS.write().await;
        *certs_lock = certs;
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_service = make_service_fn(|_conn: &AddrStream| async {
        Ok::<_, Infallible>(service_fn(proxy_handler))
    });

    println!("Reverse proxy listening on http://{}", addr);
    println!("Upstream: {}", UPSTREAM_URL.as_str());
    println!(
        "Cert path: {}",
        CERT_PATH.as_deref().unwrap_or("<system default>")
    );

    let server = Server::bind(&addr).serve(make_service);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn proxy_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let certs = GLOBAL_CERTS.read().await;
    let client = make_https_client_with_custom_cert(certs.as_deref());

    let start = Instant::now();
    let now = Local::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();

    // Add foo=bar to query
    let original_path_and_query = uri.path_and_query().map(|x| x.as_str()).unwrap_or("/");
    let new_path_and_query = if original_path_and_query.contains('?') {
        format!("{}&foo=bar", original_path_and_query)
    } else {
        format!("{}?foo=bar", original_path_and_query)
    };
    let target_uri = format!("{}{}", UPSTREAM_URL.as_str(), new_path_and_query)
        .parse::<Uri>()
        .unwrap();

    let (mut parts, body) = req.into_parts();
    parts.uri = target_uri;
    parts.headers.insert("x-custom-header", "my-value".parse().unwrap());
    let new_req = Request::from_parts(parts, body);

    let result = client.request(new_req).await;
    let duration = start.elapsed().as_millis();

    match &result {
        Ok(response) => {
            println!(
                "[{}] [{}] {}{} -> {} ({} ms)",
                now.format("%Y-%m-%d %H:%M:%S"),
                method,
                path,
                if query.is_empty() { "".to_string() } else { format!("?{}", query) },
                response.status(),
                duration
            );
        }
        Err(e) => {
            eprintln!(
                "[{}] [{}] {}{} -> ERROR: {} ({} ms)",
                now.format("%Y-%m-%d %H:%M:%S"),
                method,
                path,
                if query.is_empty() { "".to_string() } else { format!("?{}", query) },
                e,
                duration
            );
        }
    }

    let mut res = result?;
    res.headers_mut()
        .insert("x-proxy-response", "injected-by-proxy".parse().unwrap());

    Ok(res)
}
