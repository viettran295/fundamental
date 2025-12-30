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
use tokio::sync::Mutex;

use crate::{
    financial_stmt::sec_client::{ConfiguredHttpClient, SecClient},
    jobs::*,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    init_all_jobs().await;
    dotenv().expect(".env not found");

    let conf_client = ConfiguredHttpClient::new().unwrap_or_default();
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""), conf_client)));

    let app = Router::new()
        // Request latest finacial statement of company with period: quarly or yearly
        .route("/{ticker}/{period}", get(requests_handler))
        .with_state(sec_client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
