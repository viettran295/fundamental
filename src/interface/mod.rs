pub trait HttpClient<T> {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn fetch_data(&self) -> Result<T, Self::Error>;
}
