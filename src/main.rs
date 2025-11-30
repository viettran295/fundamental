mod common;
mod financial_stmt;
mod interface;

use financial_stmt::sec_client::SecClient;

use crate::{
    financial_stmt::{
        FinancialStatement, balance_sheet::BalanceSheet, cash_flow::CashFlow,
        income_statement::IncomeStatement, sec_client::ConfiguredHttpClient,
    },
    interface::HttpClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let conf_client = ConfiguredHttpClient::new()?;
    let sec_client = SecClient::new(String::from("NVDA"), conf_client);
    let json = sec_client.fetch_data().await?;

    let mut income_stmt = IncomeStatement::default();
    income_stmt.parse_quarly_latest(&json).expect("Err");

    let mut balance_sheet = BalanceSheet::default();
    balance_sheet.parse_quarly_latest(&json).expect("Err");

    let mut cash_flow = CashFlow::default();
    cash_flow.parse_annually_latest(&json).expect("Err");

    println!("{:?}", income_stmt);
    println!("{:?}", balance_sheet);
    println!("{:?}", cash_flow);

    Ok(())
}
