use std::future::Future;

use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;

pub trait HttpClient<T> {
    type Error: std::error::Error + Send + Sync + 'static + From<reqwest::Error>;

    fn fetch_data(&mut self) -> impl Future<Output = Result<T, Self::Error>> + Send;

    /// Fetch and convert json data to rust object
    fn fetch_json<RustObj: DeserializeOwned>(
        url: String,
    ) -> impl Future<Output = Result<RustObj, Self::Error>> + Send {
        async {
            let client = Self::create_client()?;
            debug!("Fetching json data {}", url);
            let response = client.get(url).send().await?.error_for_status()?;
            let json_data: RustObj = response.json().await?;
            Ok(json_data)
        }
    }

    fn create_client() -> Result<Client, Self::Error> {
        reqwest::Client::builder()
            .user_agent("(contact@example.com)")
            .build()
            .map_err(Into::into)
    }
}
