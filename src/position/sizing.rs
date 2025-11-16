use crate::config::Config;
use crate::types::ohlcv::Row;
/// Strategy for determining position size
#[derive(Debug, Clone, Copy)]
pub enum SizingStrategy {
    /// Fixed number of shares
    Fixed(i64),
    /// Fixed dollar amount
    FixedDollar(f64),
    /// Percentage of account value
    PercentOfAccount(f64),
    /// Risk-based sizing (risk % of account, requires stop loss)
    RiskBased { risk_percent: f64, stop_distance: f64 },
}

impl SizingStrategy {
    /// Calculate the number of shares to trade
    pub fn calculate(&self, row: &Row, account_value: f64) -> i64 {
        match self {
            SizingStrategy::Fixed(shares) => *shares,
            SizingStrategy::FixedDollar(amount) => {
                (amount / row.close).floor() as i64
            }
            SizingStrategy::PercentOfAccount(pct) => {
                let amount = account_value * (pct / 100.0);
                (amount / row.close).floor() as i64
            }
            SizingStrategy::RiskBased { risk_percent, stop_distance } => {
                let risk_amount = account_value * (risk_percent / 100.0);
                (risk_amount / stop_distance).floor() as i64
            }
        }
    }
}