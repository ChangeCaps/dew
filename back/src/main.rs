mod v1;

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use api::v1::Todo;
use axum::Router;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

const PORT: u16 = 7890;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let state = AppState {
        generation: AtomicU64::new(0),
        todos: Mutex::new(HashMap::new()),
    };

    let app = Router::new()
        .nest("/api/v1", v1::router())
        .with_state(Arc::new(state));

    let listener = TcpListener::bind(("0.0.0.0", PORT)).await?;
    axum::serve(listener, app).await?;

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
