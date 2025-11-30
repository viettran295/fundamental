use crate::common::MetaData;
use crate::financial_stmt::FinancialStatement;

#[derive(Debug)]
pub struct IncomeStatement {
    pub total_revenue: i64,
    pub cost_of_revenue: i64,
    pub gross_profit: i64,
    pub operating_expense: i64,
    pub operating_income: i64,
    pub net_income: i64,
    pub sec_us_gaap: [&'static str; 6],
    pub financial_facts: MetaData,
}

impl Default for IncomeStatement {
    fn default() -> Self {
        Self {
            total_revenue: 0,
            cost_of_revenue: 0,
            gross_profit: 0,
            operating_expense: 0,
            operating_income: 0,
            net_income: 0,
            sec_us_gaap: [
                "Revenues",
                "CostOfRevenue",
                "GrossProfit",
                "OperatingExpenses",
                "NonoperatingIncomeExpense",
                "NetIncomeLoss",
            ],
            financial_facts: MetaData::default(),
        }
    }
}

impl FinancialStatement for IncomeStatement {
    fn set_gaap_value(&mut self, gaap_tag: &str, value: i64) {
        match gaap_tag {
            "Revenues" => self.total_revenue = value,
            "CostOfRevenue" => self.cost_of_revenue = value,
            "GrossProfit" => self.gross_profit = value,
            "OperatingExpenses" => self.operating_expense = value,
            "NonoperatingIncomeExpense" => self.operating_income = value,
            "NetIncomeLoss" => self.net_income = value,
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
