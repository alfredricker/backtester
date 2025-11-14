use crate::types::ohlcv::{Row};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonField{
    Open,
    High,
    Low,
    Close,
    // as f64
    Volume,
    Median,
    Typical,
    WeightedClose,
}

impl CommonField {
    pub fn extract(&self, row: &Row) -> f64 {
        match self {
            CommonField::Open => row.open,
            CommonField::High => row.high,
            CommonField::Low => row.low,
            CommonField::Close => row.close,
            CommonField::Typical => (row.high + row.low + row.close) / 3.0,
            CommonField::WeightedClose => (row.high + row.low + row.close + row.close) / 4.0,
            CommonField::Median => (row.high + row.low) / 2.0,
            CommonField::Volume => row.volume as f64,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceField{
    Open,
    High,
    Low,
    Close,
    Median,
    Typical
}

impl PriceField {
    pub fn extract(&self, row: &Row) -> f64 {
        match self {
            PriceField::Open => row.open,
            PriceField::High => row.high,
            PriceField::Low => row.low,
            PriceField::Close => row.close,
            PriceField::Typical => (row.high + row.low + row.close) / 3.0,
            PriceField::Median => (row.high + row.low) / 2.0,
        }
    }
}