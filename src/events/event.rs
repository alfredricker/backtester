use crate::indicators::indicator::Indicator;
use crate::indicators::fields::CommonField;
use crate::types::ohlcv::Row;

/// Trait that all events must implement
/// Events track conditions and emit signals when those conditions are met.
/// EVENTS HAVE A CHECK METHOD THAT RETURNS A BOOL
/// They maintain internal state to detect changes (like crossovers) and 
/// can be reset to clear their history.
pub trait Event: std::fmt::Debug {
    /// Update the event with new data
    /// 
    /// Returns true if the event condition was triggered, false otherwise
    fn update(&mut self, indicators: &[Indicator], row: &Row) -> bool;

    /// Check if the event condition was triggered or confidence value was returned
    fn check(&mut self, indicators: &[Indicator], row: &Row) -> bool;
    
    /// Reset the event state (clear history)
    fn reset(&mut self);
    
    /// Get a human-readable name for the event
    fn name(&self) -> &str;
}

/// Represents a threshold value for comparison
#[derive(Debug, Clone)]
pub enum Threshold {
    /// A fixed numeric value
    Fixed(f64),
    /// Reference to another indicator by its index
    Indicator(usize),
    /// A field extracted from the current row (e.g., Close, High, Low)
    Field(CommonField),
}

impl Threshold {
    /// Get the current threshold value
    /// 
    /// Returns None if the threshold is an indicator and it doesn't have a value yet
    pub fn get_value(&self, indicators: &[Indicator], row: &Row) -> Option<f64> {
        match self {
            Threshold::Fixed(value) => Some(*value),
            Threshold::Indicator(idx) => {
                indicators.get(*idx).and_then(|ind| ind.get())
            }
            Threshold::Field(field) => Some(field.extract(row)),
        }
    }
}

