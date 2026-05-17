use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
    Router,
};
use tokio::sync::Mutex;
use tower::ServiceExt;

use fundamental::{financial_stmt::sec_client::SecClient, jobs::requests_handler};

fn build_app() -> Router {
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""))));

    Router::new()
        .route("/{ticker}/{period}", axum::routing::get(requests_handler))
        .with_state(sec_client)
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
    let (status, body) = get(build_app(), "/AAPL/annually").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
async fn test_report_quarly() {
    let (status, body) = get(build_app(), "/COIN/quarly").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
async fn test_report_history() {
    let (status, body) = get(build_app(), "/GOOG/history").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
#[should_panic]
async fn test_unknown_symbol() {
    let (status, body) = get(build_app(), "/ERROR/quarly").await;
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
    let (status, body) = get(build_app(), "/nvda/annually").await;
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
    let (status, body) = get(build_app(), "/NVDA/monthly").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "response: {:?}", body);
    let _json = parse_json(&body);
}
