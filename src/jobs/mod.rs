use std::str::FromStr;
use std::sync::Arc;

use axum::debug_handler;
use axum::extract::{Path, State};
use chrono::Utc;
use cron::Schedule;
use log::{debug, warn};
use serde_json::json;
use tokio::sync::Mutex;
use tokio::{fs, time};

use crate::common::{self, FormReport};
use crate::processor::Processor;
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

pub async fn init_all_jobs() {
    tokio::spawn(async {
        let proc = Processor::default();
        job_fetch_all_market_data(&proc).await;
    });
}

async fn job_fetch_all_market_data(data_fetcher: &impl HttpClient<serde_json::Value>) {
    // SEC update bulk zip file nightly at 3:00 a.p ET.
    // https://www.sec.gov/search-filings/edgar-application-programming-interfaces
    // This job downloads and decompresses all market data from SEC daily at 4am UTC.
    let schedule = Schedule::from_str("0 0 4 * * *").unwrap();
    let zip_file = format!(
        "{}/{}",
        common::LOCAL_DATA_STORAGE,
        common::ALL_MARKET_DATA_ZIP
    );
    for datetime in schedule.upcoming(Utc) {
        let now = Utc::now();
        if let Ok(wait) = datetime.signed_duration_since(now).to_std() {
            debug!(
                "Next job fetch_all_market_data is scheduled for: {}",
                datetime
            );
            time::sleep(wait).await;
        } else {
            continue;
        }
        debug!("Staring job: fetch_all_market_data");
        if let Err(e) = data_fetcher.fetch_data().await {
            warn!("Error fetching data in job: fetch_all_market_data: {}", e);
            continue;
        }
        if let Err(e) = common::utils::decompress_zip_file(&zip_file).await {
            warn!("Error decompressing in job: fetch_all_market_data: {}", e);
            continue;
        }
        if let Err(e) = fs::remove_file(&zip_file).await {
            warn!(
                "Error removing {} in job: fetch_all_market_data: {}",
                zip_file, e
            );
            continue;
        }
        debug!("Finished job: fetch_all_market_data");
    }
}
