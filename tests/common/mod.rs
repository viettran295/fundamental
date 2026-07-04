#![allow(dead_code)]
use log::warn;
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
    Router,
};
use testcontainers::{
    ContainerAsync, GenericImage, core::{IntoContainerPort, WaitFor}, runners::AsyncRunner,
};
use tokio::sync::Mutex;
use tower::ServiceExt;

use fundamental::common::AppState;
use fundamental::db::{dragonfly_cache::DragonFlyCache, DataManager};
use fundamental::processor::Processor;
use fundamental::{financial_stmt::sec_client::SecClient, jobs::requests_handler};

pub static CACHE_DB_IMG: &str = "docker.dragonflydb.io/dragonflydb/dragonfly";
pub static CACHE_DB_PORT: u16 = 6379;

pub async fn init_cache_db() -> (ContainerAsync<GenericImage> , String, u16) {
    let cache_db_container = GenericImage::new(CACHE_DB_IMG, "latest")
        .with_exposed_port(CACHE_DB_PORT.tcp())
        .with_wait_for(WaitFor::healthcheck())
        .start()
        .await
        .inspect_err(|e| println!("Error: {}", e)).unwrap();
    let host = cache_db_container.get_host().await.unwrap().to_string();
    let host_port = cache_db_container
        .get_host_port_ipv4(CACHE_DB_PORT)
        .await
        .unwrap();
    (cache_db_container, host, host_port)
}

pub async fn build_app() -> Router {
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""))));
    let proc = Arc::new(Mutex::new(Processor::default()));
    let (_cache_db_container, host, host_port) = init_cache_db().await;
    let uri = env::var("CACHE_DB_URI")
        .unwrap_or(format!("redis://{host}:{host_port}"))
        .to_string();
    let timeout_seconds: i64 = 60 * 60 * 24;
    let db = match DragonFlyCache::init(uri, timeout_seconds).await {
        Ok(db) => db,
        Err(e) => {
            warn!("Error connecting cache Db: {:?}", e);
            return Router::default();
        }
    };
    let shared_db = Arc::new(Mutex::new(db));
    let app_state = AppState {
        sec_client: sec_client,
        proc: proc,
        db: shared_db,
    };

    Router::new()
        .route("/{ticker}/{period}", axum::routing::get(requests_handler))
        .with_state(app_state)
}

pub async fn get(app: Router, uri: &str) -> (StatusCode, Bytes) {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, body)
}

pub fn parse_json(body: &Bytes) -> serde_json::Value {
    serde_json::from_slice(body)
        .unwrap_or_else(|e| panic!("Response is not valid JSON: {e}\nBody: {:?}", body))
}
