use serde::{Deserialize, Serialize};

use crate::common::MetaData;
use crate::financial_stmt::FinancialStatement;

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatement {
    #[serde(alias = "SalesRevenueNet")]
    #[serde(alias = "RevenueFromContractWithCustomerExcludingAssessedTax")]
    pub total_revenue: i64,
    pub cost_and_expenses: i64,
    pub cost_of_revenue: i64,
    pub gross_profit: i64,
    pub operating_expense: i64,
    #[serde(alias = "OperatingIncomeLoss")]
    pub operating_income: i64,
    pub net_income: i64,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub sec_us_gaap: [String; 11],
    pub financial_facts: MetaData,
}

impl Default for IncomeStatement {
    fn default() -> Self {
        Self {
            total_revenue: 0,
            cost_and_expenses: 0,
            cost_of_revenue: 0,
            gross_profit: 0,
            operating_expense: 0,
            operating_income: 0,
            net_income: 0,
            sec_us_gaap: [
                "Revenues".to_string(),
                // This label is equivalent to 'Revenues' in AMZN, but deprecated in 2018
                // and replaced by 'RevenueFromContractWithCustomerExcludingAssessedTax'.
                "SalesRevenueNet".to_string(),
                // This label is equivalent to 'Revenues' in AMZN.
                "RevenueFromContractWithCustomerExcludingAssessedTax".to_string(),
                // This label is in AMZN
                "CostsAndExpenses".to_string(),
                "CostOfRevenue".to_string(),
                // This label is equivalent to 'CostOfRevenues' in AMZN.
                "CostOfGoodsAndServicesSold".to_string(),
                "GrossProfit".to_string(),
                "OperatingExpenses".to_string(),
                "NonoperatingIncomeExpense".to_string(),
                // Operating income (NonoperatingIncomeExpense) for COIN
                "OperatingIncomeLoss".to_string(),
                "NetIncomeLoss".to_string(),
            ],
            financial_facts: MetaData::default(),
        }
    }
}

impl FinancialStatement for IncomeStatement {
    fn set_gaap_value(&mut self, gaap_tag: &str, value: i64) {
        match gaap_tag {
            "Revenues" => self.total_revenue = value,
            "RevenueFromContractWithCustomerExcludingAssessedTax" => self.total_revenue = value,
            "SalesRevenueNet" => self.total_revenue = value,
            "CostsAndExpenses" => self.cost_and_expenses = value,
            "CostOfRevenue" => self.cost_of_revenue = value,
            "CostOfGoodsAndServicesSold" => self.cost_of_revenue = value,
            "GrossProfit" => self.gross_profit = value,
            "OperatingExpenses" => self.operating_expense = value,
            "NonoperatingIncomeExpense" => self.operating_income = value,
            "OperatingIncomeLoss" => self.operating_income = value,
            "NetIncomeLoss" => self.net_income = value,
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
        if self.operating_expense == 0 && self.cost_and_expenses != 0 {
            self.operating_expense = self.cost_and_expenses - self.cost_of_revenue;
        }
        if self.gross_profit == 0 {
            self.gross_profit = self.total_revenue - self.cost_of_revenue;
        }
    }
}
