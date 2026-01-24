use log::warn;
use serde::{Deserialize, Serialize};

use crate::common::utils;

#[derive(Debug, Deserialize, Serialize)]
pub struct Ratios {
    pub current_ratio: f64,
    pub quick_ratio: f64,
    pub equity_ratio: f64,
    pub debt_ratio: f64,
    pub debt_to_equity_ratio: f64,
    pub gross_profit_margin: f64,
    pub operation_grofit_margin: f64,
    pub net_grofit_margin: f64,
}

impl Default for Ratios {
    fn default() -> Self {
        Self {
            current_ratio: 0.0,
            quick_ratio: 0.0,
            equity_ratio: 0.0,
            debt_ratio: 0.0,
            debt_to_equity_ratio: 0.0,
            gross_profit_margin: 0.0,
            operation_grofit_margin: 0.0,
            net_grofit_margin: 0.0,
        }
    }
}

impl Ratios {
    /// # Liquidity ratio
    /// Current ratio is a short-term liquidity.
    /// It measures a companay's ability to pay short-term obligations.
    /// Higher -> more ability to pay short-term debt.
    pub fn current_ratio(&mut self, current_assets: f64, current_liabilities: f64) {
        if current_liabilities == 0.0 {
            warn!("Error calculating current ratio: current liabilities is 0");
            self.current_ratio = 0.0;
            return;
        }
        self.current_ratio = utils::round(current_assets / current_liabilities, 2);
    }

    /// # Liquidity ratio
    /// Quick ratio is a short-term liquidity, but stricter than current ratio.
    pub fn quick_ratio(&mut self, current_assets: f64, current_liabilities: f64, inventory: f64) {
        if current_liabilities == 0.0 {
            warn!("Error calculating quick ratio: current liabilities is 0");
            self.quick_ratio = 0.0;
            return;
        }
        self.quick_ratio = utils::round((current_assets - inventory) / current_liabilities, 2);
    }

    /// # Solvency ratio
    /// It measures a company's financial stability.
    /// 30-40% is solid and healthy but it depends on the industry.
    /// Higher -> more stable.
    pub fn equity_ratio(&mut self, total_equity: f64, total_asset: f64) {
        if total_asset == 0.0 {
            warn!("Error calculating equity ratio: total asset is 0");
            self.equity_ratio = 0.0;
            return;
        }
        self.equity_ratio = utils::round(total_equity / total_asset, 2);
    }

    /// # Solvency ratio
    /// Opposite to equity ratio.
    pub fn debt_ratio(&mut self, total_liabilities: f64, total_asset: f64) {
        if total_asset == 0.0 {
            warn!("Error calculating debt ratio: total asset is 0");
            self.debt_ratio = 0.0;
            return;
        }
        self.debt_ratio = utils::round(total_liabilities / total_asset, 2);
    }

    /// # Solvency ratio
    /// It measures a company's financial leverage.
    /// Higher D/E ratio -> more risk
    pub fn debt_to_equity_ratio(&mut self, total_liabilities: f64, total_equity: f64) {
        if total_equity == 0.0 {
            warn!("Error calculating debt to equity ratio: total equity is 0");
            self.debt_to_equity_ratio = 0.0;
            return;
        }
        self.debt_to_equity_ratio = utils::round(total_liabilities / total_equity, 2);
    }

    /// # Profitability ratio
    /// Gross income ratio
    pub fn gross_profit_margin(&mut self, gross_income: f64, total_revenue: f64) {
        if total_revenue == 0.0 {
            warn!("Error calculating gross profit margin: total revenue is 0");
            self.gross_profit_margin = 0.0;
            return;
        }
        self.gross_profit_margin = utils::round(gross_income / total_revenue, 2);
    }

    /// # Profitability ratio
    /// Operating income ratio
    pub fn operating_profit_margin(&mut self, operating_income: f64, total_revenue: f64) {
        if total_revenue == 0.0 {
            warn!("Error calculating operating profit margin: total revenue is 0");
            self.operation_grofit_margin = 0.0;
            return;
        }
        self.operation_grofit_margin = utils::round(operating_income / total_revenue, 2);
    }

    /// # Profitability ratio
    /// Net income ratio
    pub fn net_profit_margin(&mut self, net_income: f64, total_revenue: f64) {
        if total_revenue == 0.0 {
            warn!("Error calculating net profit margin: total revenue is 0");
            self.net_grofit_margin = 0.0;
            return;
        }
        self.net_grofit_margin = utils::round(net_income / total_revenue, 2);
    }
}

// ---- Test ----
#[cfg(test)]
mod unittests {
    use super::*;

    #[test]
    fn test_current_ratio() {
        let current_assets = 1000.0;
        let mut current_liabilities = 5000.0;
        let mut ratio = Ratios::default();
        ratio.current_ratio(current_assets, current_liabilities);
        assert_eq!(ratio.current_ratio, (current_assets / current_liabilities));

        current_liabilities = 0.0;
        ratio.current_ratio(current_assets, current_liabilities);
        assert_eq!(ratio.current_ratio, 0.0);
    }

    #[test]
    fn test_quick_ratio() {
        let current_assets = 1000.0;
        let mut current_liabilities = 500.0;
        let inventory = 200.0;
        let mut ratio = Ratios::default();
        ratio.quick_ratio(current_assets, current_liabilities, inventory);
        assert_eq!(
            ratio.quick_ratio,
            ((current_assets - inventory) / current_liabilities)
        );
        current_liabilities = 0.0;
        ratio.quick_ratio(current_assets, current_liabilities, inventory);
        assert_eq!(ratio.quick_ratio, 0.0);
    }

    #[test]
    fn test_equity_ratio() {
        let total_equity = 1000.0;
        let mut total_asset = 500.0;
        let mut ratio = Ratios::default();
        ratio.equity_ratio(total_equity, total_asset);
        assert_eq!(ratio.equity_ratio, (total_equity / total_asset));

        total_asset = 0.0;
        ratio.equity_ratio(total_equity, total_asset);
        assert_eq!(ratio.equity_ratio, 0.0);
    }

    #[test]
    fn test_debt_ratio() {
        let total_liabilities = 500.0;
        let mut total_asset = 1000.0;
        let mut ratio = Ratios::default();
        ratio.debt_ratio(total_liabilities, total_asset);
        assert_eq!(ratio.debt_ratio, (total_liabilities / total_asset));

        total_asset = 0.0;
        ratio.debt_ratio(total_liabilities, total_asset);
        assert_eq!(ratio.debt_ratio, 0.0);
    }

    #[test]
    fn test_debt_to_equity_ratio() {
        let total_liabilities = 500.0;
        let mut total_equity = 1000.0;
        let mut ratio = Ratios::default();
        ratio.debt_to_equity_ratio(total_liabilities, total_equity);
        assert_eq!(
            ratio.debt_to_equity_ratio,
            (total_liabilities / total_equity)
        );
        total_equity = 0.0;
        ratio.debt_to_equity_ratio(total_liabilities, total_equity);
        assert_eq!(ratio.debt_to_equity_ratio, 0.0);
    }

    #[test]
    fn test_gross_profit_margin() {
        let gross_income = 500.0;
        let mut total_revenue = 1000.0;
        let mut ratio = Ratios::default();
        ratio.gross_profit_margin(gross_income, total_revenue);
        assert_eq!(ratio.gross_profit_margin, (gross_income / total_revenue));

        total_revenue = 0.0;
        ratio.gross_profit_margin(gross_income, total_revenue);
        assert_eq!(ratio.gross_profit_margin, 0.0);
    }

    #[test]
    fn test_operating_profit_margin() {
        let operating_income = 500.0;
        let mut total_revenue = 1000.0;
        let mut ratio = Ratios::default();
        ratio.gross_profit_margin(operating_income, total_revenue);
        assert_eq!(
            ratio.gross_profit_margin,
            (operating_income / total_revenue)
        );
        total_revenue = 0.0;
        ratio.gross_profit_margin(operating_income, total_revenue);
        assert_eq!(ratio.gross_profit_margin, 0.0);
    }

    #[test]
    fn test_net_profit_margin() {
        let net_income = 500.0;
        let mut total_revenue = 1000.0;
        let mut ratio = Ratios::default();
        ratio.net_profit_margin(net_income, total_revenue);
        assert_eq!(ratio.net_grofit_margin, (net_income / total_revenue));

        total_revenue = 0.0;
        ratio.net_profit_margin(net_income, total_revenue);
        assert_eq!(ratio.net_grofit_margin, 0.0);
    }
}
