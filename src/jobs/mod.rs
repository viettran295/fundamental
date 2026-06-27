use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use cron::Schedule;
use log::{debug, error, warn};
use serde_json::{json, Value};
use tokio::sync::{Mutex, Notify};
use tokio::{fs, time};

use crate::common::utils::is_dir_empty;
use crate::common::{self, utils, AppState, FormReport};
use crate::db::DataManager;
use crate::financial_stmt::{FinancialReport, FinancialReportHistory};
use crate::processor::Processor;
use crate::ratios::Ratios;
use crate::{
    financial_stmt::{
        balance_sheet::BalanceSheet, cash_flow::CashFlow, income_statement::IncomeStatement,
        sec_client::SecClient,
    },
    interface::HttpClient,
};

pub async fn requests_handler(
    State(app_state): State<AppState>,
    Path((ticker, period)): Path<(String, FormReport)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let json: Value = match utils::load_company_data(&ticker).await {
        Ok(data) => data,
        Err(e) => {
            warn!(
                "Could not load data locally -> fetching data directly from SEC - {}",
                e
            );
            let mut lock_client = app_state.sec_client.lock().await;
            lock_client.set_ticker(ticker.clone());
            lock_client.fetch_data().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error fetch data for {}: {}", ticker, e),
                )
            })?
        }
    };
    match period {
        FormReport::History => {
            let mut income_statement: Vec<IncomeStatement> = Vec::new();
            let mut balance_sheet: Vec<BalanceSheet> = Vec::new();
            let mut cash_flow: Vec<CashFlow> = Vec::new();
            utils::fill_financial_stmt_data_history(
                &mut income_statement,
                &mut balance_sheet,
                &mut cash_flow,
                &json,
            )?;
            Ok(Json(json!(FinancialReportHistory {
                balance_sheet,
                income_statement,
                cash_flow,
            })))
        }
        _ => {
            let mut income_statement = IncomeStatement::default();
            let mut balance_sheet = BalanceSheet::default();
            let mut cash_flow = CashFlow::default();
            utils::fill_financial_stmt_data(
                &mut income_statement,
                &mut balance_sheet,
                &mut cash_flow,
                &period,
                &json,
            )?;
            Ok(Json(json!(FinancialReport {
                balance_sheet,
                income_statement,
                cash_flow,
            })))
        }
    }
}

pub async fn average_ratios_requests_handler(
    State(app_state): State<AppState>,
    Path(ticker): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cik = SecClient::ticker_to_cik(&ticker).await.unwrap().unwrap();
    let trimmed_cik = cik.trim_start_matches("CIK");
    {
        let mut db_lock = app_state.db.lock().await;
        if db_lock.is_empty().await.unwrap_or(true) {
            // fixed logic
            return Err((
                StatusCode::NOT_FOUND,
                "Error fetching industry average ratios".to_string(),
            ));
        }
    }
    let sic = {
        let mut proc_lock = app_state.proc.lock().await;
        proc_lock.map_sic_to_cik().await.unwrap();
        proc_lock
            .map_cik_to_sic
            .get(trimmed_cik)
            .unwrap()
            .to_string()
    };
    let mut db_lock = app_state.db.lock().await;
    let ratios = db_lock.get(sic).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error getting average ratios: {} - {:?}", ticker, e),
        )
    })?;
    return Ok(Json(json!(Ratios::new(
        *ratios.get("current_ratio").unwrap_or(&0.0),
        *ratios.get("quick_ratio").unwrap_or(&0.0),
        *ratios.get("equity_ratio").unwrap_or(&0.0),
        *ratios.get("debt_ratio").unwrap_or(&0.0),
        *ratios.get("debt_to_equity_ratio").unwrap_or(&0.0),
        *ratios.get("gross_profit_margin").unwrap_or(&0.0),
        *ratios.get("operating_profit_margin").unwrap_or(&0.0),
        *ratios.get("net_profit_margin").unwrap_or(&0.0)
    ))));
}

pub async fn init_all_jobs(
    proc: Arc<Mutex<Processor>>,
    db: Arc<Mutex<dyn DataManager<String, HashMap<String, f64>> + Send + Sync>>,
) {
    let notifier = Arc::new(Notify::new());
    let n1 = notifier.clone();
    let n2 = notifier.clone();

    let proc1 = proc.clone();
    let proc2 = proc.clone();
    // Job: calculate industry ratio average
    tokio::spawn(async move {
        loop {
            // Wait for new data
            n2.notified().await;
            let mut db_lock = db.lock().await;
            let mut proc_lock = proc2.lock().await;
            if db_lock.is_empty().await.ok().unwrap_or(false) {
                debug!("Cache db is empty");
                job_calculate_industry_ratio_average(&mut proc_lock, &mut *db_lock).await;
            }
            drop(db_lock);
            drop(proc_lock);
        }
    });
    // Job: fetch market data
    tokio::spawn(async move {
        job_fetch_all_market_data(&proc1, n1).await;
    });
}

async fn job_fetch_all_market_data(
    data_fetcher: &impl HttpClient<serde_json::Value>,
    notifier: Arc<Notify>,
) {
    // SEC update bulk zip file nightly at 3:00 a.p ET.
    // https://www.sec.gov/search-filings/edgar-application-programming-interfaces
    // This job downloads and decompresses all market data from SEC daily at 4am UTC.
    let schedule = Schedule::from_str("0 0 4 * * *").unwrap();
    if let Err(e) = fs::create_dir_all(common::LOCAL_DATA_STORAGE).await {
        error!("Error creating local data storage: {}", e);
    }
    if let Ok(is_empty) = is_dir_empty(common::LOCAL_DATA_STORAGE) {
        if is_empty {
            // let data_fetcher_lock = data_fetcher.lock().await;
            debug!("Local data storage is empty");
            run_fetch_data(&*data_fetcher).await;
            notifier.notify_one();
        }
    }
    for datetime in schedule.upcoming(Utc) {
        let now = Utc::now();
        if let Ok(wait) = datetime.signed_duration_since(now).to_std() {
            debug!(
                "Next job fetch_all_market_data is scheduled for: {}",
                datetime
            );
            notifier.notify_one();
            time::sleep(wait).await;
        }
        if let Err(e) = fs::remove_dir(common::LOCAL_DATA_STORAGE).await {
            error!("Error cleaning up local data: {}", e);
        }
        // let data_fetcher_lock = data_fetcher.lock().await;
        run_fetch_data(&*data_fetcher).await;
        notifier.notify_one();
    }
}

// Use as public function in crate for unit tests
pub(crate) async fn run_fetch_data(data_fetcher: &impl HttpClient<serde_json::Value>) {
    let zip_files = vec![common::MARKET_DATA_ZIP, common::MARKET_META_DATA_ZIP];
    debug!("Starting job: fetch_all_market_data");
    if let Err(e) = data_fetcher.fetch_data().await {
        warn!("Error fetching data in job: fetch_all_market_data: {}", e);
        return;
    }
    for file in &zip_files {
        let zip_file = format!("{}/{}", common::LOCAL_DATA_STORAGE, file);
        // Decompress and extract SIC data
        if *file == common::MARKET_META_DATA_ZIP {
            if let Err(e) = common::utils::decompress_and_filter_sic(&zip_file).await {
                warn!("Error decompressing in job: fetch_all_market_data: {}", e);
                continue;
            }
        } else {
            if let Err(e) = common::utils::decompress_zip_file(&zip_file).await {
                warn!("Error decompressing in job: fetch_all_market_data: {}", e);
                continue;
            }
        }
        if let Err(e) = fs::remove_file(&zip_file).await {
            warn!(
                "Error removing {} in job: fetch_all_market_data: {}",
                zip_file, e
            );
            continue;
        }
    }
    debug!("Finished job: fetch_all_market_data");
}

async fn job_calculate_industry_ratio_average(
    proc: &mut Processor,
    db: &mut (impl DataManager<String, HashMap<String, f64>> + ?Sized),
) {
    debug!("Starting job: calculate_industry_ratio_average");
    if let Err(e) = proc.calculate_industry_average_of_finacial_ratios().await {
        warn!("Error calculating ratios industry average: {}", e);
    }
    // Key: SIC, value: ratio - value
    for (sic, fields) in proc.map_ratios_industry_average.clone() {
        db.set(sic, fields).await;
    }
    debug!("Finished job: calculate_industry_ratio_average");
}
