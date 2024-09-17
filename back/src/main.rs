mod v1;

use std::{
    collections::HashMap,
    env, fs, io,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use api::v1::Todo;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::Mutex, time};
use uuid::Uuid;

const PORT: u16 = 7890;
const DATA_FILE: &str = "data.ron";

#[derive(Parser)]
struct Options {
    #[clap(long, default_value = "300")]
    store_interval: u64,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let options = Options::parse();

    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let data_path = match env::var("DATA") {
        Ok(path) => PathBuf::from(path),
        Err(_) => PathBuf::from(DATA_FILE),
    };

    let state = Arc::new(AppState::load(&data_path)?);

    tokio::spawn({
        let state = state.clone();
        let store_interval = time::Duration::from_secs(options.store_interval);
        let data_path = data_path.clone();

        async move {
            loop {
                time::sleep(store_interval).await;

                tracing::info!("Storing data");

                if let Err(err) = state.store(&data_path).await {
                    tracing::error!("Failed to store data: {:?}", err);
                }
            }
        }
    });

    let app = Router::new()
        .nest("/api/v1", v1::router())
        .with_state(state);

    match (env::var("SSL_CERT"), env::var("SSL_KEY")) {
        (Ok(cert), Ok(key)) => {
            let config = RustlsConfig::from_pem_file(cert, key).await?;

            axum_server::bind_rustls(SocketAddr::from(([0; 4], PORT)), config)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            let listener = TcpListener::bind(SocketAddr::from(([0; 4], PORT))).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}

#[derive(Default, Debug)]
pub struct AppState {
    pub generation: AtomicU64,
    pub todos: Mutex<HashMap<Uuid, Todo>>,
}

impl AppState {
    pub fn load(data_path: &Path) -> eyre::Result<Self> {
        let file = match fs::File::open(data_path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(err) => eyre::bail!(err),
        };
        let data: DataOwned = ron::de::from_reader(file)?;

        match data {
            DataOwned::V1(data) => Ok(Self::from_v1(data)?),
        }
    }

    fn from_v1(data: DataV1Owned) -> eyre::Result<Self> {
        Ok(Self {
            generation: AtomicU64::new(0),
            todos: Mutex::new(data.todos),
        })
    }

    pub fn increment_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn store(&self, data_path: &Path) -> eyre::Result<()> {
        let todos = self.todos.lock().await;
        let data = DataBorrowd::V1(DataV1Borrowed { todos: &todos });

        let file = fs::File::create(data_path)?;
        let mut json = ron::Serializer::new(file, Some(Default::default()))?;
        data.serialize(&mut json)?;

        Ok(())
    }
}

#[derive(Serialize)]
enum DataBorrowd<'a> {
    V1(DataV1Borrowed<'a>),
}

#[derive(Serialize)]
struct DataV1Borrowed<'a> {
    todos: &'a HashMap<Uuid, Todo>,
}

#[derive(Deserialize)]
enum DataOwned {
    V1(DataV1Owned),
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct DataV1Owned {
    todos: HashMap<Uuid, Todo>,
}
