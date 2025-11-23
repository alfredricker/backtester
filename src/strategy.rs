use crate::position::condition::Conditionable;
use crate::types::ohlcv::Row;
use crate::position::strategy::PositionStrategy;

pub struct Strategy<L:Conditionable,R:Conditionable> {
    pub name: String,
    // Entry and exit conditions -- if no Positioning with Action::Exit, then the default is to hold until the max position time is reached,
    pub position_strategies: Vec<PositionStrategy<L,R>>,
}

impl<L:Conditionable,R:Conditionable> Strategy<L,R> {
    pub fn new(name: String, position_strategies: Vec<PositionStrategy<L,R>>) -> Self {
        Self {
            name,
            position_strategies,
        }
    }

    pub fn update(&mut self, row: &Row) {
        for s in &mut self.position_strategies {
            s.update(row);
        }
    }
}