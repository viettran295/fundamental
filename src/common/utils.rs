use log::{debug, error, warn};
use serde_json::Value;
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufReader},
    path,
};
use tokio::task;
use zip::ZipArchive;

use crate::{common, financial_stmt::sec_client::SecClient};

pub async fn decompress_zip_file(file_path: &str) -> io::Result<()> {
    let own_zip_path = file_path.to_owned();
    let result = task::spawn_blocking(move || -> io::Result<()> {
        let zip_path = path::Path::new(&own_zip_path);
        debug!("Starting to decompress {:?}", zip_path);
        // Extract and create output directory
        let unzip_dir = zip_path
            .parent()
            .unwrap_or_else(|| path::Path::new(""))
            .join(zip_path.file_stem().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid zip file name")
            })?);
        if unzip_dir.exists() && unzip_dir.is_dir() {
            fs::remove_dir_all(&unzip_dir)?;
        }
        fs::create_dir_all(&unzip_dir)?;
        // Read and loop through all zip files
        let zip_file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(zip_file)?;
        for i in 0..archive.len() {
            // Get files in .zip and combine with output directory path
            let mut compressed_file = archive.by_index(i)?;
            let compressed_file_path = match compressed_file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };
            let output_path = unzip_dir.join(&compressed_file_path);
            // Decompress file (not directory) in .zip
            if !compressed_file.name().ends_with('/') {
                let mut output_file = match fs::File::create(&output_path) {
                    Ok(file) => file,
                    Err(e) => {
                        warn!("Error creating file: {:?} - {}", &output_path, e);
                        continue;
                    }
                };
                if let Err(e) = io::copy(&mut compressed_file, &mut output_file) {
                    warn!("Error decompressing file: {:?} - {}", output_file, e);
                    continue;
                };
            }
        }
        debug!("Finish to decompress {:?}", zip_path);
        Ok(())
    })
    .await;
    match result {
        Ok(result) => result,
        Err(e) => {
            error!("Error decompressing zip file: {}", file_path);
            Err(io::Error::other(e))
        }
    }
}

/// Load all market company data locally in data/all_market_data
pub async fn load_company_data(ticker: &String) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let cik = match SecClient::ticker_to_cik(ticker).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Error converting ticker to CIK: {}", ticker);
            return Err(e.to_string().into());
        }
    };
    let mut cik = match cik {
        Some(value) => value,
        None => return Err(format!("CIK of {} is None", ticker).into()),
    };
    cik.push_str(".json");
    let all_market_data_path = format!("{}/all_market_data", common::LOCAL_DATA_STORAGE);
    if let Ok(entries) = fs::read_dir(all_market_data_path) {
        for entry in entries.flatten() {
            if entry.file_type().unwrap().is_file() && entry.file_name() == cik.as_str() {
                debug!("Found {} - {} locally", ticker, cik);
                let file = File::open(entry.path())?;
                let reader = BufReader::new(file);
                let json_data: Value = serde_json::from_reader(reader)?;
                return Ok(json_data);
            }
        }
    }
    Err("Not found".into())
}

pub fn round(num: f64, digits: u32) -> f64 {
    let mut res: f64 = 0.0;
    if digits < 1 {
        return res;
    }
    let mut base: i32 = 10;
    base = base.pow(digits);
    res = (num * base as f64).round() / base as f64;
    return res;
}
