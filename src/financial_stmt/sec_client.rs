use crate::interface::HttpClient;

use reqwest::{
    Response,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Debug)]
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
struct CompanyTickers {
    cik: u32,
    #[allow(dead_code)]
    name: Option<String>,
    ticker: Option<String>,
    #[allow(dead_code)]
    exchange: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SICResponse {
    sic: String,
    #[serde(rename = "sicDescription")]
    sic_description: String,
}

#[derive(Debug, serde::Deserialize)]
struct SecResponse {
    #[allow(dead_code)]
    pub fields: Vec<String>,
    pub data: Vec<CompanyTickers>,
}

/// Client fetches data from SEC API
#[derive(Debug)]
pub struct SecClient {
    ticker: String,
    http_client: ConfiguredHttpClient,
}

impl SecClient {
    /// SEC API endpoint for company ticker, which is used for ticker-to-CIK mapping
    const TICKER_LOOKUP_URL: &str = "https://www.sec.gov/files/company_tickers_exchange.json";
    /// Base URL for submission data
    const SUBMISSIONS_BASE_URL: &str = "https://data.sec.gov/submissions";
    /// Base URL for company facts data
    const COMPANY_FACTS_BASE_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts";

    pub fn new(ticker: String, http_client: ConfiguredHttpClient) -> Self {
        Self {
            ticker,
            http_client,
        }
    }

    /// Fetch company's metadata Standard Industry Code (SIC)
    pub async fn fetch_sic(&self) -> Result<SICResponse, Box<dyn std::error::Error>> {
        let cik = self.ticker_to_cik().await?;
        let url = format!(
            "{}/{}.json",
            Self::SUBMISSIONS_BASE_URL,
            cik.unwrap_or_default()
        );
        let data: SICResponse = self.fetch_json(&url).await?;
        Ok(data)
    }

    async fn ticker_to_cik(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let sec_response: SecResponse = self.fetch_json(Self::TICKER_LOOKUP_URL).await?;
        let company_tickers: Vec<CompanyTickers> = sec_response.data;
        for company_ticker in company_tickers {
            if company_ticker.ticker.unwrap_or(String::from("")) == self.ticker {
                return Ok(Some(format!("CIK000{}", company_ticker.cik)));
            }
        }
        Ok(None)
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
        let data = self.fetch_json(&url).await?;
        Ok(data)
    }
}
