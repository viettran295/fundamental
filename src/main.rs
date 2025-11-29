mod common;
mod financial_stmt;
mod interface;

use financial_stmt::sec_client::SecClient;

use crate::{
    financial_stmt::{income_statement::IncomeStatement, sec_client::ConfiguredHttpClient},
    interface::HttpClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let conf_client = ConfiguredHttpClient::new()?;
    let sec_client = SecClient::new(String::from("NVDA"), conf_client);
    let json = sec_client.fetch_data().await?;

    let income_stmt = IncomeStatement::default();
    income_stmt.parse_quarly_latest(&json).expect("Err");

    Ok(())
}
