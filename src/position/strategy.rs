use crate::position::sizing::SizingStrategy;
use crate::position::condition::{Condition,Conditionable};
use crate::position::order::OrderType;
use crate::types::ohlcv::Row;

// could be profit target, time based, stop loss, etc.
pub enum Action {
    Entry,
    Exit
}
pub struct PositionStrategy<L: Conditionable,R: Conditionable> {
    pub condition: Condition<L,R>, // can be built from multiple conditions
    pub sizing: SizingStrategy,
    pub order: OrderType,
    pub action: Action,
    pub name: String,
}    

impl<L: Conditionable, R:Conditionable> PositionStrategy<L,R> {
    pub fn new(condition: Condition<L,R>, sizing: SizingStrategy, order: OrderType, action: Action, name: Option<String>) -> Self {
        Self {
            condition,
            sizing,
            order,
            action,
            name : name.unwrap_or("Untitled".to_string()),
        }
    }

    pub fn update(&mut self, row: &Row) {
        self.condition.update(row);
    }
}