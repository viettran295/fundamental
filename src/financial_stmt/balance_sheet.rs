use crate::{common::MetaData, financial_stmt::FinancialStatement};

#[derive(Debug)]
pub struct BalanceSheet {
    pub total_assets: i64,
    pub current_assets: i64,
    pub inventory: i64,
    pub total_liabilities: i64,
    pub current_liabilities: i64,
    pub total_equity: i64,
    pub sec_us_gaap: [&'static str; 6],
    pub financial_facts: MetaData,
}

impl Default for BalanceSheet {
    fn default() -> Self {
        Self {
            total_assets: 0,
            current_assets: 0,
            inventory: 0,
            total_liabilities: 0,
            current_liabilities: 0,
            total_equity: 0,
            sec_us_gaap: [
                "LiabilitiesAndStockholdersEquity",
                "AssetsCurrent",
                "InventoryNet",
                "Liabilities",
                "LiabilitiesCurrent",
                "StockholdersEquity",
            ],
            financial_facts: MetaData::default(),
        }
    }
}

impl FinancialStatement for BalanceSheet {
    fn set_gaap_value(&mut self, gaap_tag: &str, value: i64) {
        match gaap_tag {
            "LiabilitiesAndStockholdersEquity" => self.total_assets = value,
            "AssetsCurrent" => self.current_assets = value,
            "InventoryNet" => self.inventory = value,
            "Liabilities" => self.total_liabilities = value,
            "LiabilitiesCurrent" => self.current_liabilities = value,
            "StockholdersEquity" => self.total_equity = value,
            _ => {}
        }
    }

    fn get_gaap_tags(&self) -> &[&'static str] {
        &self.sec_us_gaap
    }

    fn get_metadata(&mut self) -> &mut MetaData {
        &mut self.financial_facts
    }
}
