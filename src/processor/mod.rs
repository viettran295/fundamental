use crate::{
    common,
    financial_stmt::sec_client::{CompanyTickers, SecClient},
    interface::HttpClient,
};
use futures::stream::{self, StreamExt};
use log::debug;
use quick_xml::{Reader, events::Event};
use serde_json::Value;
use std::{collections::HashMap, path::Path};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Debug, Default, Clone)]
pub struct Processor {
    /// Collections of companies in each industry
    /// Key: SEC Standard industry code (SIC), value: vector of SEC Central index key (CIK)
    pub company_industry_mapping: HashMap<String, Vec<u32>>,
}

impl Processor {
    const ALL_MARKET_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";
    const MAX_CONCURRENT_REQUESTS: usize = 8;
    pub async fn map_company_by_industry(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sec_client = SecClient::default();
        let company_tickers: HashMap<String, CompanyTickers> =
            sec_client.fetch_all_company_tickers().await?;
        let results = stream::iter(company_tickers.values())
            .map(|ticker| async move {
                let mut client = SecClient::default();
                client.set_ticker(ticker.ticker.as_ref().cloned().unwrap_or_default());
                debug!("Fetching {} SIC code", ticker.ticker.as_ref().unwrap());
                let res = client.fetch_sic().await;
                (ticker, res)
            })
            .buffer_unordered(Self::MAX_CONCURRENT_REQUESTS)
            .collect::<Vec<_>>()
            .await;
        for (ticker, res) in results {
            if let Ok(xml) = res
                && let Ok(Some(sic)) = Self::extract_sic(&xml)
            {
                self.company_industry_mapping
                    .entry(sic)
                    .or_default()
                    .push(ticker.cik_str);
            }
        }
        Ok(())
    }

    fn extract_sic(xml: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut inside_sic = false;
        let mut sic: Option<String> = None;
        let sic_xml_event = b"assigned-sic";
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if e.name().as_ref() == sic_xml_event => {
                    inside_sic = true;
                }
                Event::Text(e) if inside_sic => {
                    sic = Some(e.unescape()?.to_string());
                }
                Event::End(e) if e.name().as_ref() == sic_xml_event => {
                    inside_sic = false;
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(sic)
    }
}

impl HttpClient<serde_json::Value> for Processor {
    type Error = reqwest::Error;
    // Get all companies data from SEC
    async fn fetch_data(&self) -> Result<serde_json::Value, Self::Error> {
        let client = Self::create_client()?;
        let response = client.get(Self::ALL_MARKET_DATA_URL).send().await?;
        let response = response.error_for_status()?;

        let output_path = format!(
            "{}/{}",
            common::LOCAL_DATA_STORAGE,
            common::ALL_MARKET_DATA_ZIP
        );
        let path = Path::new(&output_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.unwrap();
        }
        debug!(
            "Fetch and store all market data from SEC in {}",
            output_path
        );

        let mut file = File::create(&output_path).await.unwrap();
        let mut stream_bytes = response.bytes_stream();
        while let Some(chunk) = stream_bytes.next().await {
            let data = chunk.unwrap();
            file.write_all(&data).await.unwrap();
        }
        debug!("Finish to fetch and store all market data from SEC ",);
        Ok(Value::default())
    }
}
