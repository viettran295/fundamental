use crate::interface::HttpClient;

use log::warn;
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
pub struct CompanyTickersExchange {
    pub cik: u32,
    #[allow(dead_code)]
    pub name: Option<String>,
    pub ticker: Option<String>,
    #[allow(dead_code)]
    pub exchange: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SICResponse {
    pub cik: String,
    pub sic: String,
    pub sic_description: String,
}

#[derive(Debug, serde::Deserialize)]
struct SecResponse {
    #[allow(dead_code)]
    pub fields: Vec<String>,
    pub data: Vec<CompanyTickersExchange>,
}

/// Client fetches data from SEC API
#[derive(Debug, Default)]
pub struct SecClient {
    ticker: String,
}

impl SecClient {
    /// SEC API endpoint for company ticker, which is used for ticker-to-CIK mapping
    const TICKER_LOOKUP_URL: &str = "https://www.sec.gov/files/company_tickers_exchange.json";
    /// Base URL for company facts data
    const COMPANY_FACTS_BASE_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts";

    pub fn new(ticker: String) -> Self {
        Self { ticker }
    }

    pub fn set_ticker(&mut self, ticker: String) {
        self.ticker = ticker;
    }

    pub async fn ticker_to_cik(
        ticker: &String,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let sec_response: SecResponse =
            Self::fetch_json(Self::TICKER_LOOKUP_URL.to_string()).await?;
        let company_tickers: Vec<CompanyTickersExchange> = sec_response.data;
        for company_ticker in company_tickers {
            if company_ticker.ticker.unwrap_or(String::from("")) == *ticker {
                return Ok(Some(Self::add_cik_padding(company_ticker.cik)));
            }
        }
        Ok(None)
    }
    fn add_cik_padding(cik: u32) -> String {
        let mut nr_digits = cik.to_string().len();
        let mut padding_digits: String = String::from("");
        let standard_cik_len = 10;

        while nr_digits < standard_cik_len {
            padding_digits += "0";
            nr_digits += 1;
        }
        format!("CIK{}{}", padding_digits, cik)
    }
}

impl HttpClient<serde_json::Value> for SecClient {
    type Error = reqwest::Error;

    async fn fetch_data(&self) -> Result<Value, Self::Error> {
        let cik = Self::ticker_to_cik(&self.ticker).await.unwrap_or_else(|e| {
            warn!("Error getting CIK: {}", e);
            None
        });
        let url = format!(
            "{}/{}.json",
            Self::COMPANY_FACTS_BASE_URL,
            cik.unwrap_or_default()
        );
        let data = Self::fetch_json(url).await?;
        Ok(data)
    }
}

#[cfg(test)]
mod unittests {
    use crate::{
        common::utils,
        financial_stmt::{
            balance_sheet::BalanceSheet, cash_flow::CashFlow, income_statement::IncomeStatement,
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_ticker_to_cik() {
        let tickers = vec![
            String::from("NVDA"),
            String::from("GOOG"),
            String::from("META"),
            String::from("TSLA"),
            String::from("AAPL"),
        ];
        let ciks = vec![
            String::from("CIK0001045810"),
            String::from("CIK0001652044"),
            String::from("CIK0001326801"),
            String::from("CIK0001318605"),
            String::from("CIK0000320193"),
        ];
        for i in 0..tickers.len() {
            let cik = SecClient::ticker_to_cik(&tickers[i])
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                ciks[i], cik,
                "Failed: cik {} and ticker {} dont match",
                ciks[i], tickers[i]
            );
        }
    }

    #[tokio::test]
    async fn test_fetch_data() {
        let tickers = vec![
            String::from("COIN"),
            String::from("AAPL"),
            String::from("META"),
        ];
        for ticker in tickers {
            let sec_client = SecClient::new(ticker);
            let data = sec_client.fetch_data().await;
            assert!(data.is_ok());
        }
    }
}
