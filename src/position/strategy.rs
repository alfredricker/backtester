use crate::position::sizing::SizingStrategy;
use crate::position::condition::Condition;
use crate::position::order::OrderType;
// could be profit target, time based, stop loss, etc.
pub enum Action {
    Entry,
    Exit
}
pub struct PositionStrategy {
    pub condition: Condition, // can be built from multiple conditions
    pub sizing: SizingStrategy,
    pub order: OrderType,
    pub action: Action
}    

impl PositionStrategy {
    pub fn new(condition: Condition, sizing: SizingStrategy, order: OrderType, action: Action) -> Self {
        Self {
            condition,
            sizing,
            order,
            action,
        }
    }
}