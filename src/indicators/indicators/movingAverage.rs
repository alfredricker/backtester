use crate::indicators::trackers::{SumTracker, WindowTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::window::Window;
use crate::indicators::indicator::Indicator;
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
}

impl Indicator for MovingAverage {
    fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    fn get(&self) -> Option<f64> {
        self.tracker.get() // sumtracker get method returns the average
    }
    
    fn reset(&mut self) {
        self.tracker.clear();
    }
    
    fn name(&self) -> &str {
        "Moving Average"
    }
}