use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::equity::master::EquityTracker;
use std::collections::HashMap;

/// Main position manager that coordinates everything
pub struct PositionManager {
    /// All positions (both open and closed)
    positions: HashMap<String, Position>,
    /// Entry strategies
    entry_strategies: Vec<EntryStrategy>,
    /// Position ID counter
    next_position_id: usize,
    /// Current account value (for sizing calculations)
    account_value: f64,
}

impl PositionManager {
    /// Create a new position manager
    pub fn new(initial_account_value: f64) -> Self {
        Self {
            positions: HashMap::new(),
            entry_strategies: Vec::new(),
            next_position_id: 0,
            account_value: initial_account_value,
        }
    }
    
    /// Add an entry strategy
    pub fn add_entry_strategy(&mut self, strategy: EntryStrategy) {
        self.entry_strategies.push(strategy);
    }
    
    /// Get all open positions
    pub fn open_positions(&self) -> Vec<&Position> {
        self.positions
            .values()
            .filter(|p| p.state == PositionState::Open)
            .collect()
    }
    
    /// Get all closed positions
    pub fn closed_positions(&self) -> Vec<&Position> {
        self.positions
            .values()
            .filter(|p| p.state == PositionState::Closed)
            .collect()
    }
    
    /// Get a specific position
    pub fn get_position(&self, id: &str) -> Option<&Position> {
        self.positions.get(id)
    }
    
    /// Update account value (e.g., after P&L calculations)
    pub fn update_account_value(&mut self, new_value: f64) {
        self.account_value = new_value;
    }
    
    /// Generate a unique position ID
    fn generate_position_id(&mut self) -> String {
        let id = format!("POS_{}", self.next_position_id);
        self.next_position_id += 1;
        id
    }
    
    /// Calculate total unrealized P&L across all open positions
    pub fn total_unrealized_pnl(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.open_positions()
            .iter()
            .map(|pos| {
                current_prices
                    .get(&pos.ticker)
                    .map(|&price| pos.unrealized_pnl(price))
                    .unwrap_or(0.0)
            })
            .sum()
    }
    
    /// Calculate total realized P&L from closed positions
    pub fn total_realized_pnl(&self) -> f64 {
        self.closed_positions()
            .iter()
            .filter_map(|pos| pos.pnl())
            .sum()
    }
}

/// Actions that can result from processing a row
#[derive(Debug)]
pub enum PositionAction {
    Entry {
        position_id: String,
        ticker: String,
        side: PositionSide,
        size: i64,
        price: f64,
        timestamp: i64,
    },
    Exit {
        position_id: String,
        ticker: String,
        price: f64,
        timestamp: i64,
        pnl: f64,
    },
}