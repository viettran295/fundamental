use serde::{Deserialize, Serialize};

use crate::common::MetaData;
use crate::financial_stmt::FinancialStatement;

#[derive(Debug, Serialize, Deserialize)]
pub struct CashFlow {
    pub operating_cash_flow: i64,
    pub investing_cash_flow: i64,
    pub financing_cash_flow: i64,
    pub end_cash_flow_position: i64,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub sec_us_gaap: [String; 4],
    pub financial_facts: MetaData,
}

impl Default for CashFlow {
    fn default() -> Self {
        Self {
            operating_cash_flow: 0,
            investing_cash_flow: 0,
            financing_cash_flow: 0,
            end_cash_flow_position: 0,
            sec_us_gaap: [
                "NetCashProvidedByUsedInOperatingActivities".to_string(),
                "NetCashProvidedByUsedInInvestingActivities".to_string(),
                "NetCashProvidedByUsedInFinancingActivities".to_string(),
                "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents".to_string(),
            ],
            financial_facts: MetaData::default(),
        }
    }
}

impl FinancialStatement for CashFlow {
    fn set_gaap_value(&mut self, gaap_tag: &str, value: i64) {
        match gaap_tag {
            "NetCashProvidedByUsedInOperatingActivities" => self.operating_cash_flow = value,
            "NetCashProvidedByUsedInInvestingActivities" => self.investing_cash_flow = value,
            "NetCashProvidedByUsedInFinancingActivities" => self.financing_cash_flow = value,
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents" => {
                self.end_cash_flow_position = value
            }
            _ => {}
        }
    }

    fn get_gaap_tags(&self) -> &[String] {
        &self.sec_us_gaap
    }

    fn get_metadata(&mut self) -> &mut MetaData {
        &mut self.financial_facts
    }

    fn additional_process(&mut self) {}
}
