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
