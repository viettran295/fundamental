use log::warn;
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
    Router,
};
use tokio::sync::Mutex;
use tower::ServiceExt;

use fundamental::common::AppState;
use fundamental::db::{dragonfly_cache::DragonFlyCache, DataManager};
use fundamental::processor::Processor;
use fundamental::{financial_stmt::sec_client::SecClient, jobs::requests_handler};

async fn build_app() -> Router {
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""))));
    let proc = Arc::new(Mutex::new(Processor::default()));
    let uri = env::var("CACHE_DB_URI")
        .unwrap_or("redis://127.0.0.1:6379".to_string())
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

async fn get(app: Router, uri: &str) -> (StatusCode, Bytes) {
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

fn parse_json(body: &Bytes) -> serde_json::Value {
    serde_json::from_slice(body)
        .unwrap_or_else(|e| panic!("Response is not valid JSON: {e}\nBody: {:?}", body))
}

#[tokio::test]
async fn test_report_annually() {
    let (status, body) = get(build_app().await, "/AAPL/annually").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
async fn test_report_quarly() {
    let (status, body) = get(build_app().await, "/COIN/quarly").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
async fn test_report_history() {
    let (status, body) = get(build_app().await, "/GOOG/history").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
#[should_panic]
async fn test_unknown_symbol() {
    let (status, body) = get(build_app().await, "/ERROR/quarly").await;
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "response: {:?}",
        body
    );
    let _json = parse_json(&body);
}

#[tokio::test]
#[should_panic]
async fn test_lowercase() {
    let (status, body) = get(build_app().await, "/nvda/annually").await;
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "response: {:?}",
        body
    );
    let _json = parse_json(&body);
}

#[tokio::test]
#[should_panic]
async fn test_invalid_period() {
    let (status, body) = get(build_app().await, "/NVDA/monthly").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "response: {:?}", body);
    let _json = parse_json(&body);
}
