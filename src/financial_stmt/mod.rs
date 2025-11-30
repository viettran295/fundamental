pub mod balance_sheet;
pub mod income_statement;
pub mod sec_client;

use crate::common::{FiscalPeriod, FormReport, MetaData};

use chrono::{Datelike, Utc};
use serde_json::{Map, Value};

pub trait FinancialStatement: Sized + Default {
    /// Get GAAP tags of  financial statements
    fn get_gaap_tags(&self) -> &[&'static str];

    /// Get meta data
    fn get_metadata(&mut self) -> &mut MetaData;

    /// Set GAAP tags to struct fields
    fn set_gaap_value(&mut self, gaap_tag: &str, value: i64);

    fn parse_quarly_latest(
        &mut self,
        json_data: &Value,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let mut financial_stmt = Self::default();

        for gaap_tag in self.get_gaap_tags() {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            let latest_data = facts_data.last().unwrap();
            financial_stmt.fill_from_sec_json(latest_data, gaap_tag);
        }
        Ok(financial_stmt)
    }

    fn parse_annually_latest(
        &mut self,
        json_data: &Value,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let current_year = Utc::now().year();
        let mut financial_stmt = Self::default();

        for gaap_tag in self.get_gaap_tags() {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            for latest_data in facts_data.iter().rev() {
                if latest_data["fy"].as_i64().unwrap() == current_year as i64 {
                    financial_stmt.fill_from_sec_json(latest_data, gaap_tag);
                    break;
                }
            }
        }
        Ok(financial_stmt)
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

    /// Fill SEC response to internal FinancialStatement
    fn fill_from_sec_json(&mut self, sec_data: &Value, gaap_tag: &str) {
        let meta_data = self.get_metadata();
        meta_data.start_date = sec_data["start"].as_str().unwrap_or("None").to_owned();
        meta_data.end_date = sec_data["end"].as_str().unwrap_or("None").to_owned();
        meta_data.form_report = FormReport::from(sec_data["form"].as_str().unwrap());
        meta_data.fiscal_period = FiscalPeriod::from(sec_data["fp"].as_str().unwrap());
        self.set_gaap_value(gaap_tag, sec_data["val"].as_i64().unwrap_or(0));
    }
}
