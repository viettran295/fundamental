use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;

pub trait HttpClient<T> {
    type Error: std::error::Error + Send + Sync + 'static + From<reqwest::Error>;

    async fn fetch_data(&self) -> Result<T, Self::Error>;

    /// Fetch and convert json data to rust object
    async fn fetch_json<RustObj: DeserializeOwned>(url: &str) -> Result<RustObj, Self::Error> {
        let client = Self::create_client()?;
        debug!("Fetching json data {}", url);
        let response = client.get(url).send().await?.error_for_status()?;
        let json_data: RustObj = response.json().await?;
        Ok(json_data)
    }

    async fn fetch_xml(url: &str) -> Result<String, Self::Error> {
        let client = Self::create_client()?;
        debug!("Fetching xml data {}", url);
        let response = client.get(url).send().await?.error_for_status()?;
        let xml_data = response.text().await?;
        Ok(xml_data)
    }

    fn create_client() -> Result<Client, Self::Error> {
        reqwest::Client::builder()
            .user_agent("(contact@example.com)")
            .build()
            .map_err(Into::into)
    }
}
