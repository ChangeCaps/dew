mod v1;

use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use api::v1::Todo;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use tokio::sync::Mutex;
use uuid::Uuid;

const PORT: u16 = 7890;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let cert = env::var("SSL_CERT").unwrap_or_else(|_| String::from("/run/secrets/cert.pem"));
    let key = env::var("SSL_KEY").unwrap_or_else(|_| String::from("/run/secrets/key.pem"));

    let config = RustlsConfig::from_pem_file(cert, key).await?;

    let state = AppState {
        generation: AtomicU64::new(0),
        todos: Mutex::new(HashMap::new()),
    };

    let app = Router::new()
        .nest("/api/v1", v1::router())
        .with_state(Arc::new(state));

    axum_server::bind_rustls(SocketAddr::from(([0; 4], PORT)), config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug)]
pub struct AppState {
    pub generation: AtomicU64,
    pub todos: Mutex<HashMap<Uuid, Todo>>,
}

impl AppState {
    pub fn increment_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::Relaxed)
    }
}
