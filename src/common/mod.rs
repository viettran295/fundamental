#[derive(Debug)]
pub enum FormReport {
    INVALID,
    Quarly,
    Annually,
}

impl From<&str> for FormReport {
    fn from(s: &str) -> Self {
        match s {
            "10-Q" => FormReport::Quarly,
            "10-K" => FormReport::Annually,
            _ => FormReport::INVALID,
        }
    }
}

#[derive(Debug)]
pub enum FiscalPeriod {
    INVALID,
    Q1,
    Q2,
    Q3,
    FY,
}

impl From<&str> for FiscalPeriod {
    fn from(s: &str) -> Self {
        match s {
            "Q1" => FiscalPeriod::Q1,
            "Q2" => FiscalPeriod::Q2,
            "Q3" => FiscalPeriod::Q3,
            "FY" => FiscalPeriod::FY,
            _ => FiscalPeriod::INVALID,
        }
    }
}

#[derive(Debug)]
pub struct MetaData {
    pub accn: String,
    pub start_date: String,
    pub end_date: String,
    pub filed_date: String,
    pub frame: String,
    pub form_report: FormReport,
    pub fiscal_period: FiscalPeriod,
}

impl Default for MetaData {
    fn default() -> Self {
        Self {
            accn: String::new(),
            start_date: String::new(),
            end_date: String::new(),
            filed_date: String::new(),
            frame: String::new(),
            form_report: FormReport::Quarly,
            fiscal_period: FiscalPeriod::Q1,
        }
    }
}
