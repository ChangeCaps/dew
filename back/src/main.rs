mod v1;

use std::{
    collections::HashMap,
    env, fs, io,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use api::v1::Todo;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex, time};
use uuid::Uuid;

const PORT: u16 = 7890;
const DATA_FILE: &str = "data.ron";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let cert = env::var("SSL_CERT").unwrap_or_else(|_| String::from("/run/secrets/cert.pem"));
    let key = env::var("SSL_KEY").unwrap_or_else(|_| String::from("/run/secrets/key.pem"));

    let config = RustlsConfig::from_pem_file(cert, key).await?;
    let state = Arc::new(AppState::load()?);

    tokio::spawn({
        let state = state.clone();
        async move {
            loop {
                time::sleep(time::Duration::from_secs(300)).await;
                if let Err(err) = state.store().await {
                    tracing::error!("Failed to store data: {:?}", err);
                }
            }
        }
    });

    let app = Router::new()
        .nest("/api/v1", v1::router())
        .with_state(state);

    axum_server::bind_rustls(SocketAddr::from(([0; 4], PORT)), config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Default, Debug)]
pub struct AppState {
    pub generation: AtomicU64,
    pub todos: Mutex<HashMap<Uuid, Todo>>,
}

impl AppState {
    pub fn load() -> eyre::Result<Self> {
        let file = match fs::File::open(DATA_FILE) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(err) => eyre::bail!(err),
        };
        let data: DataOwned = ron::de::from_reader(file)?;

        match data {
            DataOwned::V1 { todos } => Ok(Self::from_v1(todos)?),
        }
    }

    fn from_v1(todos: HashMap<Uuid, Todo>) -> eyre::Result<Self> {
        Ok(Self {
            generation: AtomicU64::new(0),
            todos: Mutex::new(todos),
        })
    }

    pub fn increment_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn store(&self) -> eyre::Result<()> {
        let todos = self.todos.lock().await;
        let data = DataBorrowd::V1 { todos: &todos };

        let file = fs::File::create(DATA_FILE)?;
        let mut json = ron::Serializer::new(file, Some(Default::default()))?;
        data.serialize(&mut json)?;

        Ok(())
    }
}

#[derive(Serialize)]
enum DataBorrowd<'a> {
    V1 { todos: &'a HashMap<Uuid, Todo> },
}

#[derive(Deserialize)]
enum DataOwned {
    V1 { todos: HashMap<Uuid, Todo> },
}
