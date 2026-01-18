use std::sync::Arc;

use axum::{Router, body::Body, http::Request, routing::get};
use log::error;
use tokio::sync::Mutex;
use tower::ServiceExt;

use fundamental::{
    financial_stmt::{
        FinancialReport,
        sec_client::{ConfiguredHttpClient, SecClient},
    },
    jobs::requests_handler,
};

#[cfg(test)]
macro_rules! is_zero {
    ($symbol: expr, $name: expr, $val: expr, $vec: expr) => {
        if $val == 0 {
            $vec.push(format!("{} - {} is 0", $symbol, $name));
        }
    };
}

#[tokio::test]
async fn test_zero_fields_in_json_response() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Error)
        .is_test(true)
        .try_init();

    let conf_client = ConfiguredHttpClient::new().unwrap_or_default();
    let sec_client = Arc::new(Mutex::new(SecClient::new(String::from(""), conf_client)));
    let app = Router::new()
        .route("/{ticker}/{period}", get(requests_handler))
        .with_state(sec_client);

    let mut failed_tests: Vec<String> = Vec::new();
    let symbols: [&str; 18] = [
        "AAPL", "COIN", "NVDA", "GOOG", "MSFT", "AMZN", "PLTR", "MSTR", "JPM", "META", "TSLA",
        "AMD", "NFLX", "ORCL", "SHOP", "AVGO", "CRWD", "BRK.B"
    ];

    for symbol in symbols {
        let response = app
            .clone()
            // Skipped the network layer
            .oneshot(
                Request::builder()
                    .uri(format!("/{symbol}/quarly"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Failed to request");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        match serde_json::from_slice::<FinancialReport>(&body) {
            Ok(report) => {
                is_zero!(
                    symbol,
                    "current_assets",
                    report.balance_sheet.current_assets,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "current_liabilities",
                    report.balance_sheet.current_liabilities,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "total_assets",
                    report.balance_sheet.total_assets,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "total_equity",
                    report.balance_sheet.total_equity,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "total_liabilities",
                    report.balance_sheet.total_liabilities,
                    failed_tests
                );
                if symbol == "AMZN" {
                    is_zero!(
                        symbol,
                        "cost_and_expenses",
                        report.income_statement.cost_and_expenses,
                        failed_tests
                    );
                }
                is_zero!(
                    symbol,
                    "cost_of_revenue",
                    report.income_statement.cost_of_revenue,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "gross_profit",
                    report.income_statement.gross_profit,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "net_income",
                    report.income_statement.net_income,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "operating_expense",
                    report.income_statement.operating_expense,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "operating_income",
                    report.income_statement.operating_income,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "total_revenue",
                    report.income_statement.total_revenue,
                    failed_tests
                );

                is_zero!(
                    symbol,
                    "cash_flow_position",
                    report.cash_flow.end_cash_flow_position,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "financing_cash_flow",
                    report.cash_flow.financing_cash_flow,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "investing_cash_flow",
                    report.cash_flow.investing_cash_flow,
                    failed_tests
                );
                is_zero!(
                    symbol,
                    "operating_cash_flow",
                    report.cash_flow.operating_cash_flow,
                    failed_tests
                );
            }
            Err(e) => {
                error!("Failed deserializing: {} - {}", symbol, e);
                continue;
            }
        };
    }
    if !failed_tests.is_empty() {
        for failed_test in failed_tests {
            error!("FAILED: {}", failed_test);
        }
    }
}
