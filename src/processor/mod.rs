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
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    task::JoinSet,
};

#[derive(Debug, Default, Clone)]
pub struct Processor {
    /// Collections of companies in each industry
    /// Key: SEC Standard industry code (SIC), value: vector of SEC Central index key (CIK)
    pub map_sic_to_cik: HashMap<String, HashSet<String>>,
    pub map_ratios_industry_average: HashMap<String, HashMap<String, f64>>,
}

impl Processor {
    const MARKET_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";
    const MARKET_META_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";
    const NUM_RATIOS: usize = 8;

    /// Calculate ratios in balance sheet: current ratio, quick ratio,
    /// equity ratio, debt ratio, debt to equity ratio
    pub async fn calculate_bs_ratios_industry_average(
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
        for (k, v) in &self.map_ratios_industry_average {
            println!("Key: {}, Value: {:?}", k, v);
        }
        Ok(())
    }

    async fn map_sic_to_cik(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
                .entry(sic_response.sic)
                .or_default()
                .insert(sic_response.cik);
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
    async fn fetch_data(&self) -> Result<serde_json::Value, Self::Error> {
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
