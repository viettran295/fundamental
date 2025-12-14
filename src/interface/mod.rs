use serde::de::DeserializeOwned;

pub trait HttpClient<T> {
    type Error: std::error::Error + Send + Sync + 'static + From<reqwest::Error>;

    async fn fetch_data(&self) -> Result<T, Self::Error>;

    /// Fetch and convert json data to rust object
    async fn fetch_json<RustObj: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<RustObj, Self::Error> {
        let client = reqwest::Client::builder()
            .user_agent("(contact@example.com)")
            .build()?;
        let response = client.get(url).send().await?.error_for_status()?;
        let json_data: RustObj = response.json().await?;
        Ok(json_data)
    }
}
