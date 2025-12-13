pub struct Ratios;

impl Ratios {
    /// # Liquidity ratio
    /// Current ratio is a short-term liquidity.
    /// It measures a companay's ability to pay short-term obligations.
    /// Higher -> more ability to pay short-term debt.
    pub fn current_ratio(current_assets: f64, current_liabilities: f64) -> f64 {
        if current_liabilities == 0.0 {
            return 0.0;
        }
        current_assets / current_liabilities
    }

    /// # Liquidity ratio
    /// Quick ratio is a short-term liquidity, but stricter than current ratio.
    pub fn quick_ratio(current_assets: f64, current_liabilities: f64, inventory: f64) -> f64 {
        if current_liabilities == 0.0 {
            return 0.0;
        }
        (current_assets - inventory) / current_liabilities
    }

    /// # Solvency ratio
    /// It measures a company's financial stability.
    /// 30-40% is solid and healthy but it depends on the industry.
    /// Higher -> more stable.
    pub fn equity_ratio(total_equity: f64, total_asset: f64) -> f64 {
        if total_asset == 0.0 {
            return 0.0;
        }
        total_equity / total_asset
    }

    /// # Solvency ratio
    /// Opposite to equity ratio.
    pub fn debt_ratio(total_liabilities: f64, total_asset: f64) -> f64 {
        if total_asset == 0.0 {
            return 0.0;
        }
        total_liabilities / total_asset
    }

    /// # Solvency ratio
    /// It measures a company's financial leverage.
    /// Higher D/E ratio -> more risk
    pub fn debt_to_equity_ratio(total_liabilities: f64, total_equity: f64) -> f64 {
        if total_equity == 0.0 {
            return 0.0;
        }
        total_liabilities / total_equity
    }

    /// # Profitability ratio
    /// Gross income ratio
    pub fn gross_profit_margin(gross_income: f64, total_revenue: f64) -> f64 {
        if total_revenue == 0.0 {
            return 0.0;
        }
        gross_income / total_revenue
    }

    /// # Profitability ratio
    /// Operating income ratio
    pub fn operating_profit_margin(gross_income: f64, total_revenue: f64) -> f64 {
        if total_revenue == 0.0 {
            return 0.0;
        }
        gross_income / total_revenue
    }

    /// # Profitability ratio
    /// Net income ratio
    pub fn net_profit_margin(net_income: f64, total_revenue: f64) -> f64 {
        if total_revenue == 0.0 {
            return 0.0;
        }
        net_income / total_revenue
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
        assert_eq!(
            Ratios::current_ratio(current_assets, current_liabilities),
            (current_assets / current_liabilities)
        );
        current_liabilities = 0.0;
        assert_eq!(
            Ratios::current_ratio(current_assets, current_liabilities),
            0.0
        );
    }

    #[test]
    fn test_quick_ratio() {
        let current_assets = 1000.0;
        let mut current_liabilities = 500.0;
        let inventory = 200.0;
        assert_eq!(
            Ratios::quick_ratio(current_assets, current_liabilities, inventory),
            ((current_assets - inventory) / current_liabilities)
        );
        current_liabilities = 0.0;
        assert_eq!(
            Ratios::quick_ratio(current_assets, current_liabilities, inventory),
            0.0
        );
    }

    #[test]
    fn test_equity_ratio() {
        let total_equity = 1000.0;
        let mut total_asset = 500.0;
        assert_eq!(
            Ratios::equity_ratio(total_equity, total_asset),
            (total_equity / total_asset)
        );
        total_asset = 0.0;
        assert_eq!(Ratios::equity_ratio(total_equity, total_asset), 0.0);
    }

    #[test]
    fn test_debt_ratio() {
        let total_liabilities = 500.0;
        let mut total_asset = 1000.0;
        assert_eq!(
            Ratios::debt_ratio(total_liabilities, total_asset),
            (total_liabilities / total_asset)
        );
        total_asset = 0.0;
        assert_eq!(Ratios::debt_ratio(total_liabilities, total_asset), 0.0);
    }

    #[test]
    fn test_debt_to_equity_ratio() {
        let total_liabilities = 500.0;
        let mut total_equity = 1000.0;
        assert_eq!(
            Ratios::debt_to_equity_ratio(total_liabilities, total_equity),
            (total_liabilities / total_equity)
        );
        total_equity = 0.0;
        assert_eq!(
            Ratios::debt_to_equity_ratio(total_liabilities, total_equity),
            0.0
        );
    }

    #[test]
    fn test_gross_profit_margin() {
        let gross_income = 500.0;
        let mut total_revenue = 1000.0;
        assert_eq!(
            Ratios::gross_profit_margin(gross_income, total_revenue),
            (gross_income / total_revenue)
        );
        total_revenue = 0.0;
        assert_eq!(
            Ratios::gross_profit_margin(gross_income, total_revenue),
            0.0
        );
    }

    #[test]
    fn test_operating_profit_margin() {
        let operating_income = 500.0;
        let mut total_revenue = 1000.0;
        assert_eq!(
            Ratios::gross_profit_margin(operating_income, total_revenue),
            (operating_income / total_revenue)
        );
        total_revenue = 0.0;
        assert_eq!(
            Ratios::gross_profit_margin(operating_income, total_revenue),
            0.0
        );
    }

    #[test]
    fn test_net_profit_margin() {
        let net_income = 500.0;
        let mut total_revenue = 1000.0;
        assert_eq!(
            Ratios::net_profit_margin(net_income, total_revenue),
            (net_income / total_revenue)
        );
        total_revenue = 0.0;
        assert_eq!(
            Ratios::gross_profit_margin(net_income, total_revenue),
            0.0
        );
    }
}
