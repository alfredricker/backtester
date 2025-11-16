use chrono::NaiveDate;
use crate::position::side::Side;
use crate::position::strategy::Action;
use crate::polars::DataFrame;
use crate::position::position::Position;
use crate::position::strategy::PositionStrategy;

pub struct TradeLog {
    pub position: Position,
    pub strategy_name: String,
    pub indicator_values: Vec<f64>,
    pub pnl: f64,
    pub condition_name: String, // the name of the PositionStrategy that triggered the action
}

impl TradeLog {
    pub fn new(position: Position, position_strategy: PositionStrategy) -> Self {

    }
}