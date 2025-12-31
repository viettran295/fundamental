mod common;
mod db;
mod financial_stmt;
mod interface;
mod jobs;
mod processor;
mod ratios;

use std::sync::Arc;

use axum::{Router, routing::get};
use dotenvy::dotenv;
use log::{error, warn};
use tokio::sync::Mutex;

use crate::{
    financial_stmt::sec_client::{ConfiguredHttpClient, SecClient},
    jobs::*,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    init_all_jobs().await;
    match dotenv() {
        Ok(_) => {}
        Err(e) => warn!("Error loading file .env: {}", e),
    }

    let conf_client = ConfiguredHttpClient::new().unwrap_or_default();
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""), conf_client)));

    let app = Router::new()
        // Request latest finacial statement of company with period: quarly or yearly
        .route("/{ticker}/{period}", get(requests_handler))
        .with_state(sec_client);

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
