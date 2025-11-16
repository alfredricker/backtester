use crate::types::ohlcv::Row;

/// Trait that all indicators must implement
/// 
/// Indicators follow the update() + get() pattern for efficient streaming/backtesting:
/// - update(): Process a new data row
/// - get(): Retrieve the current indicator value (returns None if not ready)
/// - reset(): Clear all internal state
/// - name(): Human-readable identifier for this indicator
pub trait Indicator: std::fmt::Debug {
    /// Update the indicator with a new data row
    fn update(&mut self, row: &Row);
    
    /// Get the current indicator value
    /// Returns None if the indicator doesn't have enough data yet
    fn get(&self) -> Option<f64>;
    
    /// Reset the indicator state (clear all history)
    fn reset(&mut self);
    
    /// Get a human-readable name for this indicator
    fn name(&self) -> &str;
}

