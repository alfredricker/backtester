use std::collections::HashMap;
use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;

/// Context for a single ticker, holding its data and indicators
pub struct TickerContext {
    pub ticker: String,
    pub indicators: HashMap<String, Box<dyn Indicator>>,
    pub latest_row: Option<Row>,
    // Could add history buffer here if needed
}

impl TickerContext {
    pub fn new(ticker: String) -> Self {
        Self {
            ticker,
            indicators: HashMap::new(),
            latest_row: None,
        }
    }

    pub fn add_indicator(&mut self, name: &str, indicator: Box<dyn Indicator>) {
        self.indicators.insert(name.to_string(), indicator);
    }

    pub fn update(&mut self, row: &Row) {
        // Update all indicators
        for indicator in self.indicators.values_mut() {
            indicator.update(row);
        }
        self.latest_row = Some(row.clone());
    }

    pub fn get_indicator(&self, name: &str) -> Option<f64> {
        self.indicators.get(name).and_then(|ind| ind.get())
    }

    /// Get all current indicator values
    pub fn get_indicator_values(&self) -> HashMap<String, f64> {
        self.indicators
            .iter()
            .filter_map(|(name, ind)| ind.get().map(|val| (name.clone(), val)))
            .collect()
    }
    
    /// Get a mutable reference to an indicator (rarely needed by strategies, mostly for setup)
    pub fn get_indicator_mut(&mut self, name: &str) -> Option<&mut Box<dyn Indicator>> {
        self.indicators.get_mut(name)
    }
}

