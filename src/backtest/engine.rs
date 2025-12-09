use std::collections::HashMap;
use crate::types::ohlcv::Row;
use crate::strategy::Strategy;
use super::context::TickerContext;
use super::portfolio::Portfolio;
use crate::config::Config;
use crate::types::log::TradeLog;

pub struct BacktestEngine {
    pub tickers: HashMap<String, TickerContext>,
    pub portfolio: Portfolio,
    pub strategy_factory: Box<dyn Fn() -> Box<dyn Strategy>>,
    // We store a strategy instance PER ticker to handle state (like "was_long")
    pub strategies: HashMap<String, Box<dyn Strategy>>,
    pub trade_logs: Vec<TradeLog>,
}

impl BacktestEngine {
    pub fn new(_config: Config, strategy_factory: Box<dyn Fn() -> Box<dyn Strategy>>) -> Self {
        Self {
            tickers: HashMap::new(),
            portfolio: Portfolio::new(),
            strategy_factory,
            strategies: HashMap::new(),
            trade_logs: Vec::new(),
        }
    }

    pub fn process_row(&mut self, row: &Row) {
        let ticker = &row.ticker;
        
        // 1. Update Price in Portfolio
        self.portfolio.update_prices(ticker, row.close);

        // 2. Get or Create Context & Strategy
        if !self.tickers.contains_key(ticker) {
            let mut context = TickerContext::new(ticker.to_string());
            let strategy = (self.strategy_factory)();
            strategy.setup(&mut context); // Register indicators
            
            self.tickers.insert(ticker.to_string(), context);
            self.strategies.insert(ticker.to_string(), strategy);
        }

        let context = self.tickers.get_mut(ticker).unwrap();
        let strategy = self.strategies.get_mut(ticker).unwrap();

        // 3. Update Context (feeds data to indicators)
        context.update(row);

        // 4. Run Strategy Logic
        let signals = strategy.generate_signals(context);
        
        // Capture indicator values for logging
        let indicator_values = context.get_indicator_values();
        let strategy_name = strategy.name().to_string();

        // 5. Execute Signals
        for signal in signals {
            if let Some(log) = self.portfolio.process_signal(
                &signal, 
                row.close, 
                row.timestamp, 
                &indicator_values, 
                &strategy_name
            ) {
                self.trade_logs.push(log);
            }
        }
    }
}
