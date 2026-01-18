use serde::{Deserialize, Serialize};

use crate::{common::MetaData, financial_stmt::FinancialStatement};

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub total_assets: i64,
    pub current_assets: i64,
    pub inventory: i64,
    pub total_liabilities: i64,
    pub current_liabilities: i64,
    pub total_equity: i64,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub sec_us_gaap: [String; 6],
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
                "LiabilitiesAndStockholdersEquity".to_string(),
                "AssetsCurrent".to_string(),
                "InventoryNet".to_string(),
                "Liabilities".to_string(),
                "LiabilitiesCurrent".to_string(),
                "StockholdersEquity".to_string(),
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

    fn get_gaap_tags(&self) -> &[String] {
        &self.sec_us_gaap
    }

    fn get_metadata(&mut self) -> &mut MetaData {
        &mut self.financial_facts
    }

    fn additional_process(&mut self) {
        if self.total_liabilities == 0 && self.total_assets != 0 {
            self.total_liabilities = self.total_assets - self.total_equity;
        }
    }
}
