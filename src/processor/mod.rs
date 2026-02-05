use crate::{
    common,
    financial_stmt::sec_client::{CompanyTickers, SecClient},
    interface::HttpClient,
};
use futures::{io::BufWriter, stream::StreamExt};
use log::{debug, warn};
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
    const MARKET_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";
    const MARKET_META_DATA_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";
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
                fs::create_dir_all(parent).await.unwrap();
            }
            debug!(
                "Fetch {} and store all market data from SEC in {}",
                url, output_path
            );
            let mut file = File::create(&output_path).await.unwrap();
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
