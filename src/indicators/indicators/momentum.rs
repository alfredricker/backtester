use crate::indicators::trackers::{ChangeTracker, WindowTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::window::Window;
use crate::indicators::indicator::Indicator;
use crate::types::ohlcv::Row;


//big question -- how would you pass an indicator type to something like momentum?
//I believe it would have to be outside of the indicator trait or else it would be circular--but I'm not sure\
#[derive(Debug)]
pub struct Momentum {
    field: CommonField,
    tracker: ChangeTracker,
}

impl Momentum {
    pub fn new(window: Window, field: CommonField)->Self{
        Self {
            field: field,
            tracker: ChangeTracker::new(window, true) // going to use percent change for momentum
        }
    }
}

impl Indicator for Momentum {
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
        "Momentum"
    }
}