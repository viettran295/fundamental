use std::collections::HashMap;

use log::warn;

use crate::financial_stmt::sec_client::{CompanyTickers, SICResponse, SecClient};

#[derive(Debug, Default)]
pub struct Processor {
    /// Collections of companies in each industry
    /// Key: SEC Standard industry code (SIC), value: vector of SEC Central index key (CIK)
    pub company_industry_mapping: HashMap<String, Vec<u32>>,
}

impl Processor {
    pub async fn map_company_by_industry(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sec_client = SecClient::default();
        let company_tickers: HashMap<String, CompanyTickers> =
            sec_client.fetch_all_company_tickers().await?;
        for ticker in company_tickers.values() {
            sec_client.set_ticker(ticker.ticker.as_ref().cloned().unwrap_or_default());
            let sic_response: SICResponse = match sec_client.fetch_sic().await {
                Ok(response) => response,
                Err(e) => {
                    warn!("Error fetching SIC: {}", e);
                    continue;
                }
            };
            self.company_industry_mapping
                .entry(sic_response.sic)
                .and_modify(|vec| {
                    vec.push(ticker.cik_str);
                })
                .or_insert_with(|| vec![ticker.cik_str]);
        }
        Ok(())
    }
}
