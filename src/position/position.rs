use std::collections::HashMap;

/// Errors that can occur during position management
#[derive(Debug, thiserror::Error)]
pub enum PositionError {
    #[error("Position not found: {0}")]
    PositionNotFound(String),
    #[error("Ticker not found in equity tracker: {0}")]
    TickerNotFound(String),
    #[error("Invalid position size: {0}")]
    InvalidSize(i64),
    #[error("Cannot close position that is already closed")]
    AlreadyClosed,
}

/// State of a position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionState {
    /// Position is open and active
    Open,
    /// Position has been closed
    Closed,
}

/// Represents an open or closed position
#[derive(Debug)]
pub struct Position {
    /// Unique identifier for this position
    pub id: String,
    /// Ticker symbol
    pub ticker: String,
    /// Position side (Long/Short)
    pub side: PositionSide,
    /// Number of shares
    pub size: i64,
    /// Entry price
    pub entry_price: f64,
    /// Entry timestamp
    pub entry_timestamp: i64,
    /// Exit price (None if position is still open)
    pub exit_price: Option<f64>,
    /// Exit timestamp (None if position is still open)
    pub exit_timestamp: Option<i64>,
    /// Current state
    pub state: PositionState,
    /// Exit conditions for this position
    pub exit_conditions: Vec<Box<dyn Event>>,
    /// Stop loss price (optional)
    pub stop_loss: Option<f64>,
    /// Take profit price (optional)
    pub take_profit: Option<f64>,
    /// Maximum time in position (nanoseconds, optional)
    pub max_time: Option<i64>,
}

impl Position {
    /// Create a new position
    pub fn new(
        id: String,
        ticker: String,
        side: PositionSide,
        size: i64,
        entry_price: f64,
        entry_timestamp: i64,
    ) -> Self {
        Self {
            id,
            ticker,
            side,
            size,
            entry_price,
            entry_timestamp,
            exit_price: None,
            exit_timestamp: None,
            state: PositionState::Open,
            exit_conditions: Vec::new(),
            stop_loss: None,
            take_profit: None,
            max_time: None,
        }
    }
    
    /// Add an exit condition to this position
    pub fn add_exit_condition(&mut self, condition: Box<dyn Event>) {
        self.exit_conditions.push(condition);
    }
    
    /// Set stop loss price
    pub fn with_stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }
    
    /// Set take profit price
    pub fn with_take_profit(mut self, take_profit: f64) -> Self {
        self.take_profit = Some(take_profit);
        self
    }
    
    /// Set maximum time in position
    pub fn with_max_time(mut self, max_time: i64) -> Self {
        self.max_time = Some(max_time);
        self
    }
    
    /// Check if any exit conditions are met
    pub fn check_exit_conditions(
        &mut self, 
        indicators: &[Indicator], 
        row: &Row
    ) -> bool {
        // Check stop loss
        if let Some(stop_loss) = self.stop_loss {
            let should_exit = match self.side {
                PositionSide::Long => row.close <= stop_loss,
                PositionSide::Short => row.close >= stop_loss,
            };
            if should_exit {
                return true;
            }
        }
        
        // Check take profit
        if let Some(take_profit) = self.take_profit {
            let should_exit = match self.side {
                PositionSide::Long => row.close >= take_profit,
                PositionSide::Short => row.close <= take_profit,
            };
            if should_exit {
                return true;
            }
        }
        
        // Check max time
        if let Some(max_time) = self.max_time {
            if row.timestamp - self.entry_timestamp >= max_time {
                return true;
            }
        }
        
        // Check custom exit conditions
        for condition in &mut self.exit_conditions {
            if condition.check(indicators, row) {
                return true;
            }
        }
        
        false
    }
    
    /// Close the position
    pub fn close(&mut self, exit_price: f64, exit_timestamp: i64) -> Result<(), PositionError> {
        if self.state == PositionState::Closed {
            return Err(PositionError::AlreadyClosed);
        }
        
        self.exit_price = Some(exit_price);
        self.exit_timestamp = Some(exit_timestamp);
        self.state = PositionState::Closed;
        Ok(())
    }
    
    /// Calculate profit/loss for this position
    pub fn pnl(&self) -> Option<f64> {
        self.exit_price.map(|exit_price| {
            let price_diff = match self.side {
                PositionSide::Long => exit_price - self.entry_price,
                PositionSide::Short => self.entry_price - exit_price,
            };
            price_diff * self.size as f64
        })
    }
    
    /// Calculate profit/loss percentage
    pub fn pnl_percent(&self) -> Option<f64> {
        self.exit_price.map(|exit_price| {
            let price_diff = match self.side {
                PositionSide::Long => exit_price - self.entry_price,
                PositionSide::Short => self.entry_price - exit_price,
            };
            (price_diff / self.entry_price) * 100.0
        })
    }
    
    /// Get unrealized P&L based on current price
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let price_diff = match self.side {
            PositionSide::Long => current_price - self.entry_price,
            PositionSide::Short => self.entry_price - current_price,
        };
        price_diff * self.size as f64
    }
}