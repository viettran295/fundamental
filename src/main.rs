mod common;
mod db;
mod financial_stmt;
mod interface;
mod jobs;
mod processor;
mod ratios;

use std::sync::Arc;

use axum::{routing::get, Router};
use dotenvy::dotenv;
use log::{error, warn};
use tokio::sync::Mutex;

use crate::common::AppState;
use crate::db::{dragonfly_cache::DragonFlyCache, DataManager};
use crate::processor::Processor;
use crate::{financial_stmt::sec_client::SecClient, jobs::*};

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(_) => {}
        Err(e) => warn!("Error loading file .env: {}", e),
    }
    env_logger::init();

    let uri = env::var("CACHE_DB_URI")
        .unwrap_or("redis://127.0.0.1:6379".to_string())
        .to_string();
    // Timeout for cache db in seconds.
    // New data is fetched every 24 hours -> Calculate industry average, cache timeout for industry average is 24 hours.
    let timeout_seconds: i64 = 60 * 60 * 24;
    let proc = Arc::new(Mutex::new(Processor::default()));
    let db = match DragonFlyCache::init(uri.clone(), timeout_seconds).await {
        Ok(db) => db,
        Err(e) => {
            error!("Error init cache db at {}: {:?}", uri, e);
            return;
        }
    };
    let shared_db = Arc::new(Mutex::new(db));
    init_all_jobs(proc.clone(), shared_db.clone()).await;
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""))));
    let app_state = AppState {
        sec_client: sec_client,
        proc: proc,
        db: shared_db,
    };

    let app = Router::new()
        .route("/{ticker}/ratios", get(average_ratios_requests_handler))
        // Request latest finacial statement of company with period: quarly or yearly
        .route("/{ticker}/{period}", get(requests_handler))
        .with_state(app_state);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Error connecting TCP: {}", e);
            return;
        }
    };

    match axum::serve(listener, app).await {
        Ok(_) => {}
        Err(e) => {
            error!("Error initializing server: {}", e);
            return;
        }
    };
}
