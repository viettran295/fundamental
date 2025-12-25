pub mod balance_sheet;
pub mod cash_flow;
pub mod income_statement;
pub mod sec_client;

use crate::common::{FiscalPeriod, FormReport, MetaData};

use chrono::{Datelike, Utc};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub struct StatementHistory<T> {
    pub records: Vec<T>,
}

impl<T: FinancialStatement> StatementHistory<T> {
    pub fn default() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    pub fn fill_history(&mut self, json_data: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = T::default();
        self.records = tmp.parse_history(json_data)?;
        Ok(())
    }
}

pub trait FinancialStatement: Default {
    const MAX_HISTORY_YEARS: usize = 5;
    /// Get GAAP tags of  financial statements
    fn get_gaap_tags(&self) -> &[&'static str];

    /// Get meta data
    fn get_metadata(&mut self) -> &mut MetaData;

    /// Set GAAP tags to struct fields
    fn set_gaap_value(&mut self, gaap_tag: &str, value: i64);

    fn parse_quarly_latest(&mut self, json_data: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let gaap_tags = self.get_gaap_tags().to_vec();
        for gaap_tag in gaap_tags {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            let latest_data = facts_data.last().unwrap();
            self.fill_from_sec_json(latest_data, gaap_tag);
        }
        Ok(())
    }

    fn parse_annually_latest(
        &mut self,
        json_data: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let current_year = Utc::now().year();
        let gaap_tags = self.get_gaap_tags().to_vec();
        for gaap_tag in gaap_tags {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            for latest_data in facts_data.iter().rev() {
                if latest_data["fy"].as_i64().unwrap() == current_year as i64 {
                    self.fill_from_sec_json(latest_data, gaap_tag);
                    break;
                }
            }
        }
        Ok(())
    }

    fn parse_history(
        &mut self,
        json_data: &Value,
    ) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let facts = Self::extract_us_gaap(json_data)?;
        let gaap_tags = self.get_gaap_tags().to_vec();
        let current_year = Utc::now().year();
        let cutoff_year = current_year - Self::MAX_HISTORY_YEARS as i32;
        let mut history: BTreeMap<&str, Self> = BTreeMap::new();

        for gaap_tag in gaap_tags {
            let facts_data = Self::extract_gaap_tag_in_unit_usd(facts, gaap_tag)?;
            for data in facts_data.iter().rev() {
                if data["form"] != "10-K" {
                    continue;
                }
                let end_date = data["end"].as_str().unwrap_or_default();
                let year_prefix = end_date.split('-').next().unwrap_or("");
                if let Ok(report_year) = year_prefix.parse::<i32>() {
                    if report_year > cutoff_year {
                        let entry = history.entry(end_date).or_insert_with(|| Self::default());
                        entry.fill_from_sec_json(data, gaap_tag);
                    } else {
                        break;
                    }
                } else {
                    continue;
                }
            }
        }
        Ok(history.into_values().rev().collect())
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

// --- Test ---
#[cfg(test)]
mod unittests {
    use super::*;
    use serde_json::json;

    #[derive(Debug, Default)]
    struct MockIncomeStatement {
        metadata: MetaData,
    }

    impl FinancialStatement for MockIncomeStatement {
        fn get_gaap_tags(&self) -> &[&'static str] {
            &["Revenues", "NetIncomeLoss"]
        }

        fn get_metadata(&mut self) -> &mut MetaData {
            &mut self.metadata
        }

        fn set_gaap_value(&mut self, _gaap_tag: &str, _value: i64) {}
    }

    fn create_mock_sec_json(current_year: i32) -> Value {
        json!({
            "facts": {
                "us-gaap": {
                    "Revenues": {
                        "units": {
                            "USD": [
                                {"val": 2000, "form": "10-Q", "fp": "Q4", "fy": current_year - 1, "start": "2024-10-01", "end": "2024-12-31"},
                                {"val": 3000, "form": "10-Q", "fp": "Q1", "fy": current_year, "start": "2025-01-01", "end": "2025-03-31"},
                            ]
                        }
                    },
                    "NetIncomeLoss": {
                        "units": {
                            "USD": [
                                {"val": 50, "form": "10-K", "fp": "FY", "fy": current_year - 1, "start": "2024-01-01", "end": "2024-12-31"},
                                {"val": 150, "form": "10-K", "fp": "FY", "fy": current_year, "start": "2025-01-01", "end": "2025-12-31"},
                            ]
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn test_extract_us_gaap() {
        let mock_obj = MockIncomeStatement::default();
        let current_year = Utc::now().year();
        let json_data = create_mock_sec_json(current_year);
        let result = MockIncomeStatement::extract_us_gaap(&json_data);
        assert!(result.is_ok());
        for tag in mock_obj.get_gaap_tags() {
            assert!(result.as_ref().unwrap().contains_key(*tag));
        }
    }

    #[test]
    fn test_extract_us_gaap_missing() {
        let json_data = json!({"facts": {}});
        let result = MockIncomeStatement::extract_us_gaap(&json_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parse error: could not find 'facts'"
        );
    }

    #[test]
    fn extract_gaap_tag_in_unit_usd() {
        let mock_obj = MockIncomeStatement::default();
        let current_year = Utc::now().year();
        let json_data = create_mock_sec_json(current_year);
        let facts = MockIncomeStatement::extract_us_gaap(&json_data);
        assert!(facts.is_ok());

        for gaap_tag in mock_obj.get_gaap_tags() {
            let facts_data = MockIncomeStatement::extract_gaap_tag_in_unit_usd(
                facts.as_ref().unwrap(),
                gaap_tag,
            );
            assert!(facts_data.is_ok());
            assert_eq!(facts_data.unwrap().len(), 2);
        }
    }

    #[test]
    fn test_extract_gaap_tag_in_unit_usd_missing_tag() {
        let json_data = create_mock_sec_json(2025);
        let facts = MockIncomeStatement::extract_us_gaap(&json_data).unwrap();
        let result = MockIncomeStatement::extract_gaap_tag_in_unit_usd(facts, "MissingTag");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parse error: could not find MissingTag"
        );
    }
}
