use crate::position::condition::{Condition,Conditionable};
use crate::position::side::Side;
use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
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
}