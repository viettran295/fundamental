use std::sync::Arc;

use axum::debug_handler;
use axum::extract::{Path, State};
use serde_json::json;
use tokio::sync::Mutex;

use crate::common::FormReport;
use crate::{
    financial_stmt::{
        FinancialStatement, balance_sheet::BalanceSheet, cash_flow::CashFlow,
        income_statement::IncomeStatement, sec_client::SecClient,
    },
    interface::HttpClient,
};

#[debug_handler]
pub async fn requests_handler(
    State(sec_client): State<Arc<Mutex<SecClient>>>,
    Path((ticker, period)): Path<(String, FormReport)>,
) -> String {
    let mut lock_client = sec_client.lock().await;
    lock_client.set_ticker(ticker);
    let json = match lock_client.fetch_data().await {
        Ok(data) => data,
        Err(_) => return String::new(),
    };

    let mut income_stmt = IncomeStatement::default();
    let mut balance_sheet = BalanceSheet::default();
    let mut cash_flow = CashFlow::default();
    match period {
        FormReport::Annually => {
            if let Err(e) = income_stmt.parse_annually_latest(&json) {
                return format!("Parsing error: {}", e);
            }
            if let Err(e) = balance_sheet.parse_annually_latest(&json) {
                return format!("Parsing error: {}", e);
            }
            if let Err(e) = cash_flow.parse_annually_latest(&json) {
                return format!("Parsing error: {}", e);
            }
        }
        FormReport::Quarly => {
            if let Err(e) = income_stmt.parse_quarly_latest(&json) {
                return format!("Parsing error: {}", e);
            }
            if let Err(e) = balance_sheet.parse_quarly_latest(&json) {
                return format!("Parsing error: {}", e);
            }
            if let Err(e) = cash_flow.parse_quarly_latest(&json) {
                return format!("Parsing error: {}", e);
            }
        }
        FormReport::Invalid => return format!("Invalid request period. Use annually or quarly."),
    }
    let response = json!({
        "balance_sheet": balance_sheet,
        "income_statement": income_stmt,
        "cash_flow": cash_flow
    });
    response.to_string()
}
