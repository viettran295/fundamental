use crate::interface::HttpClient;

use log::warn;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
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
    pub fn new(ticker: String, http_client: ConfiguredHttpClient) -> Self {
        Self {
            ticker,
            http_client,
        }
    }

    async fn ticker_to_cik(search_ticker: &str) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let url = "https://www.sec.gov/files/company_tickers_exchange.json";
        let client = reqwest::Client::builder()
            .user_agent("(contact@example.com)")
            .build()?;
        let response = client.get(url).send().await?.error_for_status()?;

        let sec_response: SecResponse = response.json().await?;
        let company_tickers: Vec<CompanyTickers> = sec_response.data;
        for company_ticker in company_tickers {
            if company_ticker.ticker.unwrap_or(String::from("")) == search_ticker {
                return Ok(Some(company_ticker.cik));
            }
        }
        Ok(None)
    }
}

impl HttpClient<serde_json::Value> for SecClient {
    type Error = reqwest::Error;

    async fn fetch_data(&self) -> Result<Value, Self::Error> {
        let cik = match Self::ticker_to_cik(&self.ticker).await {
            Ok(Some(found_cik)) => {
                format!("CIK000{}", found_cik)
            }
            Ok(None) | Err(_) => {
                warn!("Ticker not fount CIK");
                String::from("")
            }
        };
        let url = format!("https://data.sec.gov/api/xbrl/companyfacts/{}.json", cik);
        let client = self.http_client.client();
        let response = client.get(url).send().await?;
        let response = response.error_for_status()?;
        let data = response.json().await?;
        Ok(data)
    }
}
