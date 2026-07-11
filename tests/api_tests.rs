mod common;
use axum::http::StatusCode;
use common::*;

#[tokio::test]
async fn test_report_annually() {
    let (status, body) = get(build_app(None).await, "/AAPL/annually").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let _json = parse_json(&body);
}

#[tokio::test]
async fn test_report_quarly() {
    let (status, body) = get(build_app(None).await, "/COIN/quarly").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let report: serde_json::Value = parse_json(&body);
    assert_report_ok(report);
}

#[tokio::test]
async fn test_report_history() {
    let (status, body) = get(build_app(None).await, "/GOOG/history").await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let report = parse_json(&body);
    assert_report_ok(report);
}

#[tokio::test]
async fn test_unknown_symbol() {
    let (status, body) = get(build_app(None).await, "/ERROR/quarly").await;
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "response: {:?}",
        body
    );
}

#[tokio::test]
async fn test_lowercase() {
    let (status, body) = get(build_app(None).await, "/nvda/annually").await;
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "response: {:?}",
        body
    );
}

#[tokio::test]
async fn test_invalid_period() {
    let (status, body) = get(build_app(None).await, "/NVDA/monthly").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "response: {:?}", body);
}

fn assert_report_ok(report: serde_json::Value) {
    assert_ne!(report["balance_sheet"]["current_assets"], 0.0);
    assert_ne!(report["balance_sheet"]["current_liabilities"], 0.0);
    assert_ne!(report["balance_sheet"]["total_assets"], 0.0);
    assert_ne!(report["balance_sheet"]["total_equity"], 0.0);
    assert_ne!(report["balance_sheet"]["total_liabilities"], 0.0);
    assert_ne!(report["cash_flow"]["end_cash_flow_position"], 0.0);
    assert_ne!(report["cash_flow"]["financing_cash_flow"], 0.0);
    assert_ne!(report["cash_flow"]["investing_cash_flow"], 0.0);
    assert_ne!(report["cash_flow"]["operating_cash_flow"], 0.0);
    assert_ne!(report["income_statement"]["total_revenue"], 0.0);
    assert_ne!(report["income_statement"]["gross_profit"], 0.0);
    assert_ne!(report["income_statement"]["operating_expense"], 0.0);
    assert_ne!(report["income_statement"]["operating_income"], 0.0);
}
