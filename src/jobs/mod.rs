use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use chrono::Utc;
use cron::Schedule;
use log::{debug, warn};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::{fs, time};

use crate::common::{self, FormReport, utils};
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let json: Value = match utils::load_company_data(&ticker).await {
        Ok(data) => data,
        Err(_) => {
            let mut lock_client = sec_client.lock().await;
            lock_client.set_ticker(ticker.clone());
            lock_client.fetch_data().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error fetch data for {}: {}", ticker, e),
                )
            })?
        }
    };
    let mut income_stmt = IncomeStatement::default();
    let mut balance_sheet = BalanceSheet::default();
    let mut cash_flow = CashFlow::default();
    match period {
        FormReport::Annually => {
            income_stmt
                .parse_annually_latest(&json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
            balance_sheet
                .parse_annually_latest(&json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
            cash_flow
                .parse_annually_latest(&json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
        }
        FormReport::Quarly => {
            income_stmt
                .parse_quarly_latest(&json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
            balance_sheet
                .parse_quarly_latest(&json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
            cash_flow
                .parse_quarly_latest(&json)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)))?;
        }
        FormReport::Invalid => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid request period. Use annually or quarly.".to_owned(),
            ));
        }
    }
    Ok(Json(json!({
        "balance_sheet": balance_sheet,
        "income_statement": income_stmt,
        "cash_flow": cash_flow
    })))
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
