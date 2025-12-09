use std::collections::HashMap;
use serde::Serialize;
use crate::position::side::Side;
use crate::indicators::indicator::Indicator;
use crate::types::ohlcv::Row;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PositionState {
    /// Position is open and active
    Open,
    /// Position has been closed
    Closed,
}

/// Represents an open or closed position
#[derive(Debug, Serialize, Clone)]
pub struct Position {
    /// Unique identifier for this position
    pub id: String,
    /// Ticker symbol
    pub ticker: String,
    /// Position side (Long/Short)
    pub side: Side,
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
    pub state: PositionState
}

impl Position {
    /// Create a new position
    pub fn new(
        id: String,
        ticker: String,
        side: Side,
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
        }
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
                Side::Long => exit_price - self.entry_price,
                Side::Short => self.entry_price - exit_price,
                Side::None => 0.0,
            };
            price_diff * self.size as f64
        })
    }
    
    /// Calculate profit/loss percentage
    pub fn pnl_percent(&self) -> Option<f64> {
        self.exit_price.map(|exit_price| {
            let price_diff = match self.side {
                Side::Long => exit_price - self.entry_price,
                Side::Short => self.entry_price - exit_price,
                Side::None => 0.0,
            };
            (price_diff / self.entry_price) * 100.0
        })
    }
    
    /// Get unrealized P&L based on current price
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let price_diff = match self.side {
            Side::Long => current_price - self.entry_price,
            Side::Short => self.entry_price - current_price,
            Side::None => 0.0,
        };
        price_diff * self.size as f64
    }
}