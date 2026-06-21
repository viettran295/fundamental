pub mod utils;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{db::DataManager, financial_stmt::sec_client::SecClient, processor::Processor};

pub const LOCAL_DATA_STORAGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
pub const MARKET_DATA_ZIP: &str = "market_data.zip";
pub const MARKET_META_DATA_ZIP: &str = "market_meta_data.zip";
pub const MARKET_META_DATA_JSON: &str = "market_meta_data.json";

#[derive(Clone)]
pub struct AppState {
    pub sec_client: Arc<Mutex<SecClient>>,
    pub proc: Arc<Mutex<Processor>>,
    pub db: Arc<Mutex<dyn DataManager<String, HashMap<String, f64>> + Send + Sync>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormReport {
    Invalid,
    Quarly,
    Annually,
    History,
}

impl From<&str> for FormReport {
    fn from(s: &str) -> Self {
        match s {
            "10-Q" => FormReport::Quarly,
            "10-K" => FormReport::Annually,
            _ => FormReport::Invalid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FiscalPeriod {
    Invalid,
    Q1,
    Q2,
    Q3,
    FY,
}

impl From<&str> for FiscalPeriod {
    fn from(s: &str) -> Self {
        match s {
            "Q1" => FiscalPeriod::Q1,
            "Q2" => FiscalPeriod::Q2,
            "Q3" => FiscalPeriod::Q3,
            "FY" => FiscalPeriod::FY,
            _ => FiscalPeriod::Invalid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaData {
    #[allow(unused)]
    pub accn: String,
    pub start_date: String,
    pub end_date: String,
    #[allow(unused)]
    pub filed_date: String,
    #[allow(unused)]
    pub frame: String,
    pub form_report: FormReport,
    pub fiscal_period: FiscalPeriod,
}

impl Default for MetaData {
    fn default() -> Self {
        Self {
            accn: String::new(),
            start_date: String::new(),
            end_date: String::new(),
            filed_date: String::new(),
            frame: String::new(),
            form_report: FormReport::Quarly,
            fiscal_period: FiscalPeriod::Q1,
        }
    }
}
