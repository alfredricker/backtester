use std::collections::HashMap;
use serde::Serialize;
use crate::position::strategy::Action;
use crate::position::position::Position;

#[derive(Serialize)]
pub struct TradeLog {
    #[serde(flatten)]
    pub position: Position,
    pub action: Action,
    pub strategy_name: String,
    pub indicator_values: HashMap<String, f64>,
    pub pnl: f64,
    pub condition_name: String, // the name of the PositionStrategy that triggered the action
}

impl TradeLog {
    pub fn new(
        position: Position,
        action: Action,
        strategy_name: String,
        condition_name: String,
        indicator_values: HashMap<String, f64>,
    ) -> Self {
        let pnl = position.pnl().unwrap_or(0.0);
        Self {
            position,
            action,
            strategy_name,
            indicator_values,
            pnl,
            condition_name,
        }
    }
}
