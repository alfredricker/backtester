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
