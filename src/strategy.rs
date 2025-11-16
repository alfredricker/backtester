use crate::position::condition::Condition;
use crate::position::side::Side;
use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;

pub struct Strategy {
    pub name: String,
    /// Entry condition - when true, open a position
    pub entry: Condition,
    /// Exit condition - when true, close the position
    pub exit: Option<Condition>,
    /// Position side (Long/Short)
    pub side: Side,
}

impl Strategy {
    pub fn new(name: String, entry: Condition, side: Side) -> Self {
        Self {
            name,
            entry,
            exit: None,
            side,
        }
    }
    
    /// Check if entry conditions are met
    pub fn check_entry(&self, indicators: &[Indicator], row: &Row) -> bool {
        self.entry.evaluate(indicators, row)
    }
    
    /// Check if exit conditions are met
    pub fn check_exit(&self, indicators: &[Indicator], row: &Row) -> bool {
        self.exit
            .as_ref()
            .map(|cond| cond.evaluate(indicators, row))
            .unwrap_or(false)
    }
}