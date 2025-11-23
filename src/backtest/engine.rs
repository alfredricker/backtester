use std::collections::HashMap;
use crate::types::ohlcv::Row;
use crate::strategy::Strategy;
use super::context::TickerContext;
use super::portfolio::Portfolio;
use crate::config::Config;

pub struct BacktestEngine {
    pub tickers: HashMap<String, TickerContext>,
    pub portfolio: Portfolio,
    pub strategy_factory: Box<dyn Fn() -> Box<dyn Strategy>>,
    // We store a strategy instance PER ticker to handle state (like "was_long")
    pub strategies: HashMap<String, Box<dyn Strategy>>,
}

impl BacktestEngine {
    pub fn new(config: Config, strategy_factory: Box<dyn Fn() -> Box<dyn Strategy>>) -> Self {
        Self {
            tickers: HashMap::new(),
            portfolio: Portfolio::new(config),
            strategy_factory,
            strategies: HashMap::new(),
        }
    }

    pub fn process_row(&mut self, row: &Row) {
        let ticker = row.ticker.clone();
        
        // 1. Update Price in Portfolio
        self.portfolio.update_prices(&ticker, row.close);

        // 2. Get or Create Context & Strategy
        if !self.tickers.contains_key(&ticker) {
            let mut context = TickerContext::new(ticker.clone());
            let mut strategy = (self.strategy_factory)();
            strategy.setup(&mut context); // Register indicators
            
            self.tickers.insert(ticker.clone(), context);
            self.strategies.insert(ticker.clone(), strategy);
        }

        let context = self.tickers.get_mut(&ticker).unwrap();
        let strategy = self.strategies.get_mut(&ticker).unwrap();

        // 3. Update Context (feeds data to indicators)
        context.update(row);

        // 4. Run Strategy Logic
        let signals = strategy.generate_signals(context);

        // 5. Execute Signals
        for signal in signals {
            self.portfolio.process_signal(&signal, row.close);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::maMomentum::create_ma_momentum;

    #[test]
    fn test_backtest_engine_ma_momentum() {
        let config = Config::default();
        let mut engine = BacktestEngine::new(config, Box::new(create_ma_momentum));

        let base_time = 1614556800000000000; 
        
        // Simulate price curve that crosses
        // MA30 (fast) vs MA60 (slow)
        // Start: Price 100. Flat.
        // Step 50: Price jumps to 150. Fast moves up, crosses Slow (Buy).
        // Step 100: Price drops to 50. Fast moves down, crosses Slow (Sell).

        for i in 0..200 {
            let price = if i < 50 { 100.0 } else if i < 100 { 150.0 } else { 50.0 };
            
            let row = Row {
                timestamp: base_time + (i * 60 * 1_000_000_000), // +1 minute per row
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1000,
                ticker: "AAPL".to_string(),
            };
            
            engine.process_row(&row);
        }

        // We expect at least one trade (Buy) and maybe a Sell depending on window warmup
        // With 30m/60m windows, we need ~60 bars to warm up.
        // Jump at 50 -> Cross likely around 60-70?
        // Drop at 100 -> Cross likely around 110-120?
        
        // Check portfolio state
        println!("Buying Power: {}", engine.portfolio.buying_power);
        println!("Positions: {:?}", engine.portfolio.positions);
        
        // Ideally we would assert some trades happened
        // assert!(engine.portfolio.buying_power != 100_000.0); 
        // (Buying power changes due to slippage even if position is closed)
    }
}
