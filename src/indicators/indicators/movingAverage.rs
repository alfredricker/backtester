use crate::indicators::trackers::{SumTracker, WindowTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::window::Window;
use crate::types::ohlcv::Row;

/// Moving Average using stateful tracking
#[derive(Debug)]
pub struct MovingAverage {
    tracker: SumTracker,
    field: CommonField,
}

impl MovingAverage {
    pub fn new(window: Window, field: CommonField) -> Self {
        Self {
            tracker: SumTracker::new(window),
            field,
        }
    }
    
    pub fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    pub fn get(&self) -> Option<f64> {
        self.tracker.get() // sumtracker get method returns the average
    }
    
    pub fn reset(&mut self) {
        self.tracker.clear();
    }
}