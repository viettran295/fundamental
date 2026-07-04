mod common;
use common::*;
use axum::http::StatusCode;

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
