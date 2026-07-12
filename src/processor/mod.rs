use crate::{
    common,
    financial_stmt::{
        balance_sheet::BalanceSheet, income_statement::IncomeStatement, sec_client::SICResponse,
        FinancialStatement,
    },
    interface::HttpClient,
    ratios::Ratios,
};
use futures::stream::StreamExt;
use log::{debug, error, warn};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
    task::JoinSet,
};

pub struct Processor {
    /// Collections of companies in each industry
    /// Key: SEC Standard industry code (SIC), value: vector of SEC Central index key (CIK)
    pub map_sic_to_cik: HashMap<String, HashSet<String>>,
    pub map_cik_to_sic: HashMap<String, String>,
    pub map_ratios_industry_average: HashMap<String, HashMap<String, f64>>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            map_cik_to_sic: HashMap::new(),
            map_sic_to_cik: HashMap::new(),
            map_ratios_industry_average: HashMap::new(),
        }
    }
}

impl Processor {
    const MARKET_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";
    const MARKET_META_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";
    const NUM_RATIOS: usize = 8;

    /// Calculate industry average of liquidity ratio, solvency ratio and profitability ratio
    pub async fn calculate_industry_average_of_finacial_ratios(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.map_sic_to_cik.is_empty() {
            self.map_sic_to_cik().await?;
        }
        for (sic, cik_s) in &self.map_sic_to_cik {
            let mut set: JoinSet<Option<[f64; Self::NUM_RATIOS]>> = JoinSet::new();
            // current, quick, equity, debt, d_to_e
            let mut sums = [0.0f64; Self::NUM_RATIOS];
            let mut elements = [0; Self::NUM_RATIOS];
            for cik in cik_s {
                let cik = cik.clone();
                set.spawn(async move {
                    let data: Value = Self::get_data_from_json(&cik).await.ok()?;

                    let mut bs = BalanceSheet::default();
                    let mut ic = IncomeStatement::default();
                    let mut ratios = Ratios::default();

                    if bs.parse_annually_latest(&data).is_ok() {
                        // Liquidity ratio
                        ratios
                            .current_ratio(bs.current_assets as f64, bs.current_liabilities as f64);
                        ratios.quick_ratio(
                            bs.current_assets as f64,
                            bs.current_liabilities as f64,
                            bs.inventory as f64,
                        );

                        // Solvency ratio
                        ratios.equity_ratio(bs.total_equity as f64, bs.total_assets as f64);
                        ratios.debt_ratio(bs.total_liabilities as f64, bs.total_assets as f64);
                        ratios.debt_to_equity_ratio(
                            bs.total_liabilities as f64,
                            bs.total_equity as f64,
                        );
                    }
                    if ic.parse_annually_latest(&data).is_ok() {
                        // Profitability ratio
                        ratios.gross_profit_margin(ic.gross_profit as f64, ic.total_revenue as f64);
                        ratios.operating_profit_margin(
                            ic.operating_income as f64,
                            ic.total_revenue as f64,
                        );
                        ratios.net_profit_margin(ic.net_income as f64, ic.total_revenue as f64);
                    }
                    Some([
                        ratios.current_ratio,
                        ratios.quick_ratio,
                        ratios.equity_ratio,
                        ratios.debt_ratio,
                        ratios.debt_to_equity_ratio,
                        ratios.gross_profit_margin,
                        ratios.operating_grofit_margin,
                        ratios.net_grofit_margin,
                    ])
                });
            }
            while let Some(res) = set.join_next().await {
                if let Ok(Some(ratios)) = res {
                    for i in 0..Self::NUM_RATIOS {
                        if ratios[i] == 0.0 {
                            continue;
                        }
                        sums[i] += ratios[i];
                        elements[i] += 1;
                    }
                }
            }
            self.map_ratios_industry_average
                .entry(sic.clone())
                .or_default()
                .extend([
                    ("current_ratio".into(), sums[0] / elements[0] as f64),
                    ("quick_ratio".into(), sums[1] / elements[1] as f64),
                    ("equity_ratio".into(), sums[2] / elements[2] as f64),
                    ("debt_ratio".into(), sums[3] / elements[3] as f64),
                    ("debt_to_equity_ratio".into(), sums[4] / elements[4] as f64),
                    ("gross_profit_margin".into(), sums[5] / elements[5] as f64),
                    (
                        "operating_profit_margin".into(),
                        sums[6] / elements[6] as f64,
                    ),
                    ("net_profit_margin".into(), sums[7] / elements[7] as f64),
                ]);
        }
        Ok(())
    }

    pub async fn map_sic_to_cik(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.map_cik_to_sic.is_empty() && !self.map_sic_to_cik.is_empty() {
            return Ok(());
        }
        let path = format!(
            "{}/{}",
            common::LOCAL_DATA_STORAGE,
            common::MARKET_META_DATA_JSON
        );
        let data = tokio::fs::read_to_string(path)
            .await
            .inspect_err(|e| warn!("{}", e))?;
        let sic_responses: Vec<SICResponse> =
            serde_json::from_str(&data).inspect_err(|e| warn!("{}", e))?;
        for sic_response in sic_responses {
            self.map_sic_to_cik
                .entry(sic_response.sic.clone())
                .or_default()
                .insert(sic_response.cik.clone());
            self.map_cik_to_sic
                .insert(sic_response.cik, sic_response.sic);
        }
        Ok(())
    }

    async fn get_data_from_json(cik: &String) -> Result<Value, Box<dyn std::error::Error>> {
        let json_path = format!("{}/market_data/CIK{}.json", common::LOCAL_DATA_STORAGE, cik);
        let mut file = match tokio::fs::File::open(&json_path).await {
            Err(e) => {
                error!("Error opening {}: {}", json_path, e);
                return Err(Box::new(e));
            }
            Ok(f) => f,
        };
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.map_err(|e| {
            error!("Error opening {}: {}", json_path, e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;
        let data: Value = serde_json::from_slice(&buffer).map_err(|e| {
            error!("Error opening {}: {}", json_path, e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;
        Ok(data)
    }
}

impl HttpClient<serde_json::Value> for Processor {
    type Error = reqwest::Error;
    // Get all companies data from SEC
    async fn fetch_data(&mut self) -> Result<serde_json::Value, Self::Error> {
        let client = Self::create_client()?;
        let url_local_file_map: HashMap<&str, &str> = HashMap::from([
            (Self::MARKET_DATA_URL, common::MARKET_DATA_ZIP),
            (Self::MARKET_META_DATA_URL, common::MARKET_META_DATA_ZIP),
        ]);
        for (url, local_file_name) in url_local_file_map {
            let response = client.get(url).send().await?;
            let response = response.error_for_status()?;
            let output_path = format!("{}/{}", common::LOCAL_DATA_STORAGE, local_file_name);
            let path = Path::new(&output_path);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await.unwrap();
            }
            debug!(
                "Fetch {} and store all market data from SEC in {}",
                url, output_path
            );
            let mut file = match tokio::fs::File::create(&output_path).await {
                Ok(file) => file,
                Err(e) => {
                    warn!("Error creating {}: {}", output_path, e);
                    continue;
                }
            };
            let mut stream_bytes = response.bytes_stream();
            while let Some(chunk) = stream_bytes.next().await {
                let data = chunk?;
                if let Err(e) = file.write_all(&data).await {
                    warn!("Error writing data to file: {}", e);
                    continue;
                }
            }
            debug!(
                "Finish to fetch {} and store all market data from SEC in {}",
                url, output_path
            );
        }
        Ok(Value::default())
    }
}

impl HttpClient<serde_json::Value> for Arc<Mutex<Processor>> {
    type Error = reqwest::Error;
    async fn fetch_data(&mut self) -> Result<serde_json::Value, Self::Error> {
        self.lock().await.fetch_data().await
    }
}

// ---- Test ----
#[cfg(test)]
mod unittests {
    use super::*;
    use crate::{common::utils, jobs};
    use tokio::fs;
    use tokio::sync::OnceCell;

    static DATA_LOCAL: OnceCell<()> = OnceCell::const_new();

    // Fetch SEC data for local tests
    async fn setup_local_data() {
        DATA_LOCAL
            .get_or_init(|| async {
                // Create local data dir if it doesn't exist, then fetch
                fs::create_dir_all(common::LOCAL_DATA_STORAGE)
                    .await
                    .expect("Error creating local data storage");
                let mut proc = Processor::default();
                if utils::is_dir_empty(common::LOCAL_DATA_STORAGE).unwrap() {
                    jobs::run_fetch_data(&mut proc).await;
                }
            })
            .await;
    }

    #[tokio::test]
    async fn test_map_sic_to_cik() {
        setup_local_data().await;

        let nvidia_cik = String::from("0001045810");
        let intel_cik = String::from("0000050863");
        let tsmc_cik = String::from("0001046179");
        let meta_cik = String::from("0001326801");
        let goog_cik = String::from("0001652044");

        let semiconductors_sic = String::from("3674");
        let computer_programming_sic = String::from("7370");

        let mut proc = Processor::default();
        proc.map_sic_to_cik().await.ok();

        let semiconductor_companies = proc.map_sic_to_cik.get(&semiconductors_sic).unwrap();
        let computer_programming_companies =
            proc.map_sic_to_cik.get(&computer_programming_sic).unwrap();

        let nvidia_sic = proc.map_cik_to_sic.get(&nvidia_cik).unwrap();
        let goog_sic = proc.map_cik_to_sic.get(&goog_cik).unwrap();

        assert_eq!(*nvidia_sic, semiconductors_sic);
        assert_eq!(*goog_sic, computer_programming_sic);

        assert!(
            semiconductor_companies.contains(&nvidia_cik),
            "Failed: Nvidia CIK missing"
        );
        assert!(
            semiconductor_companies.contains(&intel_cik),
            "Failed: Intel CIK missing"
        );
        assert!(
            semiconductor_companies.contains(&tsmc_cik),
            "Failed: Qualcomm CIK missing"
        );
        assert!(
            computer_programming_companies.contains(&meta_cik),
            "Failed: Meta CIK missing"
        );
        assert!(
            computer_programming_companies.contains(&goog_cik),
            "Failed: Google CIK missing"
        );
    }

    #[tokio::test]
    async fn test_get_data_from_json() {
        setup_local_data().await;

        let nvidia_cik = String::from("0001045810");
        let intel_cik = String::from("0000050863");
        let goog_cik = String::from("0001652044");
        let companies_cik = vec![nvidia_cik, intel_cik, goog_cik];

        for cik in companies_cik {
            let data = Processor::get_data_from_json(&cik).await.ok().unwrap();
            let mut bs = BalanceSheet::default();
            bs.parse_annually_latest(&data).ok();
            assert_ne!(
                bs.total_assets, 0,
                "Failed: total assets of {} not equal to 0",
                cik
            );
            assert_ne!(
                bs.total_liabilities, 0,
                "Failed: total liabilities of {} not equal to 0",
                cik
            );
        }
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_calculate_industry_average_of_financial_ratios() {
        setup_local_data().await;

        let semiconductors_sic = String::from("3674");
        let oil_sic = String::from("3533");
        let computer_storage_sic = String::from("3572");
        let sics = [semiconductors_sic, oil_sic, computer_storage_sic];
        let ratios = [
            "current_ratio",
            "quick_ratio",
            "equity_ratio",
            "debt_ratio",
            "debt_to_equity_ratio",
            "gross_profit_margin",
            "operating_profit_margin",
            "net_profit_margin",
        ];
        let mut proc = Processor::default();
        proc.calculate_industry_average_of_finacial_ratios()
            .await
            .ok();
        for sic in sics {
            let averages = proc
                .map_ratios_industry_average
                .get(&sic)
                .expect(format!("Failed: can not get {} SIC", sic).as_str());
            for ratio in ratios {
                let value = averages.get(&ratio.to_string()).unwrap();
                assert!(value.is_finite(), "Failed: {} is not finite", ratio);
                assert_ne!(*value, 0.0, "Failed: {} ratio average is 0", ratio);
            }
        }
    }
}
