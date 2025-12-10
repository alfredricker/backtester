use crate::config::Config;
use crate::types::ohlcv::Row;
use crate::backtest::signal::Signal;
/// Strategy for determining position size
#[derive(Debug, Clone, Copy)]
pub enum SizingStrategy {
    /// Fixed number of shares
    Fixed(i64),
    /// Fixed dollar amount
    FixedDollar(f64),
    /// Percentage of account value (buying power)
    PercentOfAccount(f64),
    /// Risk-based sizing (risk % of account, requires stop loss)
    RiskBased { risk_percent: f64, stop_distance: f64 },
    /// signal based, pass function that takes in signal and outputs f64
    SignalBased(fn(Signal) -> f64),
}

impl SizingStrategy {
    /// Calculate the number of shares to trade
    pub fn calculate(&self, price: f64, account_value: f64, _signal: Option<&Signal>) -> i64 {
        match self {
            SizingStrategy::Fixed(shares) => *shares,
            SizingStrategy::FixedDollar(amount) => {
                (amount / price).floor() as i64
            }
            SizingStrategy::PercentOfAccount(pct) => {
                let amount = account_value * (pct / 100.0);
                (amount / price).floor() as i64
            }
            SizingStrategy::RiskBased { risk_percent, stop_distance } => {
                let risk_amount = account_value * (risk_percent / 100.0);
                (risk_amount / (price * (1.0 - stop_distance))).floor() as i64
            }
            SizingStrategy::SignalBased(func) => {
                0 // @TODO: implement this
            }
        }
    }
}