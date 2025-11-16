use crate::position::condition::Condition;
use crate::position::side::Side;
use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::position::strategy::PositionStrategy;

pub struct Strategy {
    pub name: String,
    // Entry and exit conditions -- if no Positioning with Action::Exit, then the default is to hold until the max position time is reached,
    pub position_strategies: Vec<PositionStrategy>,
}

impl Strategy {
    pub fn new(name: String, position_strategies: Vec<PositionStrategy>) -> Self {
        Self {
            name,
            position_strategies,
        }
    }
}