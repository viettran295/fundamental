use crate::financial_stmt::sec_client::{CompanyTickers, SecClient};
use futures::stream::{self, StreamExt};
use log::debug;
use quick_xml::{Reader, events::Event};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Processor {
    /// Collections of companies in each industry
    /// Key: SEC Standard industry code (SIC), value: vector of SEC Central index key (CIK)
    pub company_industry_mapping: HashMap<String, Vec<u32>>,
}

impl Processor {
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
            if let Ok(xml) = res {
                if let Ok(Some(sic)) = Self::extract_sic(&xml) {
                    self.company_industry_mapping
                        .entry(sic)
                        .or_default()
                        .push(ticker.cik_str);
                }
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
