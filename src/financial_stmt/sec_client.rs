use std::collections::HashMap;

use crate::interface::HttpClient;

use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ConfiguredHttpClient {
    client: reqwest::Client,
}

impl ConfiguredHttpClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let user_agent_value = "(example@example.com)";
        let header_value = HeaderValue::from_str(user_agent_value)
            .expect("Error: User-Agent string literal should always be valid");
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, header_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(ConfiguredHttpClient { client })
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CompanyTickersExchange {
    pub cik: u32,
    #[allow(dead_code)]
    pub name: Option<String>,
    pub ticker: Option<String>,
    #[allow(dead_code)]
    pub exchange: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CompanyTickers {
    pub cik_str: u32,
    pub ticker: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SICResponse {
    pub sic: String,
    #[serde(rename = "sicDescription")]
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
    http_client: ConfiguredHttpClient,
}

impl SecClient {
    /// SEC API endpoint for company ticker, which is used for ticker-to-CIK mapping
    const TICKER_LOOKUP_URL: &str = "https://www.sec.gov/files/company_tickers_exchange.json";
    /// Simplified version of TICKER_LOOKUP_URL
    const COMPANY_TICKERS_SIMPLIFIED: &str = "https://www.sec.gov/files/company_tickers.json";
    /// Base URL for submission data
    const SUBMISSIONS_BASE_URL: &str = "https://data.sec.gov/submissions";
    /// Base URL for company facts data
    const COMPANY_FACTS_BASE_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts";
    const SIC_BASE_URL: &str =
        "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&output=atom";

    pub fn new(ticker: String, http_client: ConfiguredHttpClient) -> Self {
        Self {
            ticker,
            http_client,
        }
    }

    pub fn set_ticker(&mut self, ticker: String) {
        self.ticker = ticker;
    }

    /// Fetch company's metadata Standard Industry Code (SIC)
    pub async fn fetch_sic(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cik = self.ticker_to_cik().await?;
        let url = format!(
            "{}&CIK={}",
            Self::SIC_BASE_URL,
            cik.unwrap_or_default()
                .strip_prefix("CIK")
                .unwrap_or_default()
        );
        debug!("Fetching SIC");
        let data = Self::fetch_xml(&url).await?;
        Ok(data)
    }

    pub async fn fetch_all_company_tickers(
        &self,
    ) -> Result<HashMap<String, CompanyTickers>, Box<dyn std::error::Error>> {
        type CompanyMap = HashMap<String, CompanyTickers>;
        debug!("Fetching all company tickers");
        let json_response: CompanyMap = Self::fetch_json(Self::COMPANY_TICKERS_SIMPLIFIED).await?;
        Ok(json_response)
    }

    async fn ticker_to_cik(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let sec_response: SecResponse = Self::fetch_json(Self::TICKER_LOOKUP_URL).await?;
        let company_tickers: Vec<CompanyTickersExchange> = sec_response.data;
        for company_ticker in company_tickers {
            if company_ticker.ticker.unwrap_or(String::from("")) == self.ticker {
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
        let cik = self.ticker_to_cik().await.unwrap_or_default();
        let url = format!(
            "{}/{}.json",
            Self::COMPANY_FACTS_BASE_URL,
            cik.unwrap_or_default()
        );
        let data = Self::fetch_json(&url).await?;
        Ok(data)
    }
}
