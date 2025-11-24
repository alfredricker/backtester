use crate::backtest::context::TickerContext;
use crate::backtest::signal::Signal;
use crate::types::ohlcv::Row;
use crate::config::Config;

/// Trait for implementing trading strategies
pub trait Strategy {
    /// Define which indicators this strategy needs
    /// This allows the engine to pre-populate the context
    fn setup(&self, context: &mut TickerContext);

    /// Generate trading signals based on the current context
    fn generate_signals(&mut self, context: &TickerContext) -> Vec<Signal>;
    
    /// Human-readable name
    fn name(&self) -> &str;
}
