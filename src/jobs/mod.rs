use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use chrono::Utc;
use cron::Schedule;
use log::{debug, error, warn};
use serde_json::{Value, json};
use tokio::sync::{Mutex, Notify};
use tokio::{fs, time};

use crate::common::utils::is_dir_empty;
use crate::common::{self, FormReport, utils};
use crate::db::{DataManager, dragonfly_cache::DragonFlyCache};
use crate::financial_stmt::FinancialReport;
use crate::processor::Processor;
use crate::ratios::Ratios;
use crate::{
    financial_stmt::{
        balance_sheet::BalanceSheet, cash_flow::CashFlow, income_statement::IncomeStatement,
        sec_client::SecClient,
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
        Err(e) => {
            warn!(
                "Could not load data locally -> fetching data directly from SEC - {}",
                e
            );
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

pub async fn ratios_requests_handler(
    State(sec_client): State<Arc<Mutex<SecClient>>>,
    Path((ticker, period)): Path<(String, FormReport)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let json: Value = match utils::load_company_data(&ticker).await {
        Ok(data) => data,
        Err(e) => {
            warn!(
                "Could not load data locally -> fetching data directly from SEC - {}",
                e
            );
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
    utils::fill_financial_stmt_data(
        &mut income_stmt,
        &mut balance_sheet,
        &mut cash_flow,
        &period,
        &json,
    )?;
    let mut ratios = Ratios::default();
    ratios.current_ratio(
        balance_sheet.current_assets as f64,
        balance_sheet.current_liabilities as f64,
    );
    ratios.quick_ratio(
        balance_sheet.current_assets as f64,
        balance_sheet.current_liabilities as f64,
        balance_sheet.inventory as f64,
    );
    ratios.equity_ratio(
        balance_sheet.total_equity as f64,
        balance_sheet.total_assets as f64,
    );
    ratios.debt_ratio(
        balance_sheet.total_liabilities as f64,
        balance_sheet.total_assets as f64,
    );
    ratios.debt_to_equity_ratio(
        balance_sheet.total_liabilities as f64,
        balance_sheet.total_equity as f64,
    );
    ratios.gross_profit_margin(
        income_stmt.gross_profit as f64,
        income_stmt.total_revenue as f64,
    );
    ratios.operating_profit_margin(
        income_stmt.operating_income as f64,
        income_stmt.total_revenue as f64,
    );
    ratios.net_profit_margin(
        income_stmt.net_income as f64,
        income_stmt.total_revenue as f64,
    );

    Ok(Json(json!(ratios)))
}

pub async fn init_all_jobs() {
    let notifier = Arc::new(Notify::new());
    let n1 = notifier.clone();
    let n2 = notifier.clone();
    tokio::spawn(async {
        let proc = Processor::default();
        job_fetch_all_market_data(&proc, n1).await;
    });
    tokio::spawn(async move {
        let mut proc = Processor::default();
        let uri = env::var("CACHE_DB_URI")
            .unwrap_or("redis://127.0.0.1:6379".to_string())
            .to_string();
        let mut db = match DragonFlyCache::init(uri).await {
            Ok(db) => db,
            Err(e) => {
                warn!("Error connecting cache Db: {:?}", e);
                return;
            }
        };
        loop {
            // Wait for new data
            n2.notified().await;
            job_calculate_industry_ratio_average(&mut proc, &mut db).await;
        }
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
            debug!("Local data storage is empty");
            run_fetch_data(data_fetcher).await;
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
            time::sleep(wait).await;
        } else {
            continue;
        }
        run_fetch_data(data_fetcher).await;
        notifier.notify_one();
    }
}

async fn run_fetch_data(data_fetcher: &impl HttpClient<serde_json::Value>) {
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
    db: &mut impl DataManager<String, HashMap<String, f64>>,
) {
    debug!("Starting job: calculate_industry_ratio_average");
    if let Err(e) = proc.map_sic_to_cik().await {
        warn!("Error mapping SIC to CIK: {}", e);
    }
    if let Err(e) = proc.calculate_bs_ratios_industry_average().await {
        warn!("Error calculating ratios industry average: {}", e);
    }
    for (sic, fields) in proc.map_ratios_industry_average.clone() {
        db.set(sic, fields).await;
    }
    debug!("Finished job: calculate_industry_ratio_average");
}
