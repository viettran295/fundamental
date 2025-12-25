mod common;
mod financial_stmt;
mod interface;
mod processor;
mod ratios;

use crate::{
    financial_stmt::{
        FinancialStatement, StatementHistory,
        balance_sheet::BalanceSheet,
        cash_flow::CashFlow,
        income_statement::IncomeStatement,
        sec_client::{ConfiguredHttpClient, SecClient},
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

    let mut bs_history = StatementHistory::<BalanceSheet>::default();
    bs_history.fill_history(&json).expect("Err");
    for rec in bs_history.records {
        println!("{:?}", rec);
        println!("-------")
    }

    let mut ic_history = StatementHistory::<IncomeStatement>::default();
    ic_history.fill_history(&json).expect("Err");
    for rec in ic_history.records {
        println!("{:?}", rec);
        println!("-------")
    }

    let mut cf_history = StatementHistory::<CashFlow>::default();
    cf_history.fill_history(&json).expect("Err");
    for rec in cf_history.records {
        println!("{:?}", rec);
        println!("-------")
    }

    Ok(())
}
