use crate::common::{FiscalPeriod, FormReport, MetaData};

use chrono::{Datelike, Utc};
use serde_json::{Map, Value};

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

impl IncomeStatement {
    pub fn set_gaap_value(&mut self, gaap_tag: &str, value: i64) {
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
    pub fn parse_quarly_latest(
        &self,
        json_data: &Value,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let mut income_stmt = Self::default();

        for gaap_tag in self.sec_us_gaap {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            let latest_data = facts_data.last().unwrap();
            Self::parse_sec_reponse_to_income_stmt(latest_data, &mut income_stmt, gaap_tag);
        }
        println!("{:?}", income_stmt);
        Ok(income_stmt)
    }

    pub fn parse_annually_latest(
        &self,
        json_data: &Value,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let mut income_stmt = Self::default();
        let current_year = Utc::now().year();

        for gaap_tag in self.sec_us_gaap {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            for latest_data in facts_data.iter().rev() {
                if latest_data["fy"].as_i64().unwrap() == current_year as i64 {
                    Self::parse_sec_reponse_to_income_stmt(latest_data, &mut income_stmt, gaap_tag);
                    break;
                }
            }
        }
        Ok(income_stmt)
    }

    /// Extract field 'us-gaap' from SEC json raw response
    fn extract_us_gaap(
        json_data: &Value,
    ) -> Result<&Map<String, Value>, Box<dyn std::error::Error>> {
        Ok(json_data["facts"]["us-gaap"]
            .as_object()
            .ok_or_else(|| String::from("Parse error: could not find 'facts'"))?)
    }

    /// Extract gaap tag in USD from 'us-gaap' field
    fn extract_gaap_tag_in_unit_usd<'a>(
        facts: &'a Map<String, Value>,
        gaap_tag: &str,
    ) -> Result<&'a Vec<Value>, Box<dyn std::error::Error>> {
        let data = facts
            .get(gaap_tag)
            .ok_or_else(|| String::from(&format!("Parse error: could not find {}", gaap_tag)))?;
        Ok(data["units"]["USD"]
            .as_array()
            .ok_or_else(|| String::from("Parse error: could not find USD unit"))?)
    }

    /// Parse SEC data response to internal IncomeStatement
    fn parse_sec_reponse_to_income_stmt(
        sec_data: &Value,
        income_stmt: &mut IncomeStatement,
        gaap_tag: &str,
    ) {
        income_stmt.financial_facts.start_date =
            sec_data["start"].as_str().unwrap_or("None").to_owned();
        income_stmt.financial_facts.end_date =
            sec_data["end"].as_str().unwrap_or("None").to_owned();
        income_stmt.financial_facts.form_report =
            FormReport::from(sec_data["form"].as_str().unwrap());
        income_stmt.financial_facts.fiscal_period =
            FiscalPeriod::from(sec_data["fp"].as_str().unwrap());
        income_stmt.set_gaap_value(gaap_tag, sec_data["val"].as_i64().unwrap_or(0));
    }
}
