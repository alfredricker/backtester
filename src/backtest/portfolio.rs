use std::collections::HashMap;
use crate::config;
use crate::position::side::Side;
use crate::position::position::Position;
use crate::types::log::TradeLog;
use crate::backtest::signal::{Signal, SignalType};
use crate::position::order::OrderType;
use crate::position::strategy::Action;
use uuid::Uuid;

pub struct Portfolio {
    pub buying_power: f64,
    pub open_positions: HashMap<String, Position>, // Ticker -> Position
    pub closed_positions: Vec<Position>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            buying_power: config::get_config().starting_buying_power,
            open_positions: HashMap::new(),
            closed_positions: Vec::new(),
        }
    }

    pub fn update_prices(&mut self, _ticker: &str, _price: f64) {
        // In this simple model, we don't store current price in Position struct persistently.
        // We could track unrealized PnL here if needed.
    }

    pub fn process_signal(
        &mut self, 
        signal: &Signal, 
        price: f64, 
        timestamp: i64, 
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str,
    ) -> Option<TradeLog> {
        match &signal.signal_type {
            SignalType::Trigger(order_type) => self.handle_trigger(order_type, &signal.ticker, price, timestamp, indicator_values, strategy_name),
            SignalType::Value(_) => None,
        }
    }

    fn handle_trigger(
        &mut self,
        order_type: &OrderType,
        ticker: &str,
        price: f64,
        timestamp: i64,
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str,
    ) -> Option<TradeLog> {
        match order_type {
            OrderType::MarketBuy() => self.handle_market_buy(ticker, price, timestamp, indicator_values, strategy_name),
            OrderType::MarketSell() => self.handle_market_sell(ticker, price, timestamp, indicator_values, strategy_name),
            _ => {
                println!("Order type not implemented in backtest portfolio: {:?}", order_type);
                None
            }
        }
    }

    fn handle_market_buy(
        &mut self,
        ticker: &str,
        price: f64,
        timestamp: i64,
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str,
    ) -> Option<TradeLog> {
        if let Some(pos) = self.open_positions.remove(ticker) {
            match pos.side {
                Side::Short => self.close_position(pos, price, timestamp, indicator_values, strategy_name),
                Side::Long => {
                    self.open_positions.insert(ticker.to_string(), pos);
                    None
                }
                Side::None => None,
            }
        } else {
            self.open_position(ticker, Side::Long, price, timestamp, indicator_values, strategy_name)
        }
    }

    fn handle_market_sell(
        &mut self,
        ticker: &str,
        price: f64,
        timestamp: i64,
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str,
    ) -> Option<TradeLog> {
         if let Some(pos) = self.open_positions.remove(ticker) {
            match pos.side {
                Side::Long => self.close_position(pos, price, timestamp, indicator_values, strategy_name),
                Side::Short => {
                    self.open_positions.insert(ticker.to_string(), pos);
                    None
                }
                Side::None => None,
            }
        } else {
            self.open_position(ticker, Side::Short, price, timestamp, indicator_values, strategy_name)
        }
    }
    
    fn close_position(
        &mut self, 
        mut pos: Position, 
        price: f64, 
        timestamp: i64,
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str
    ) -> Option<TradeLog> {
        if let Ok(_) = pos.close(price, timestamp) {
            // Update BP
            match pos.side {
                Side::Short => {
                    let cost = price * pos.size as f64;
                    self.buying_power -= cost;
                }
                Side::Long => {
                    let proceeds = price * pos.size as f64;
                    self.buying_power += proceeds;
                }
                _ => {}
            }
            
            let log = TradeLog::new(
                pos.clone(),
                Action::Exit,
                strategy_name.to_string(),
                "Signal".to_string(),
                indicator_values.clone()
            );
            self.closed_positions.push(pos);
            Some(log)
        } else {
            None
        }
    }

    fn open_position(
        &mut self,
        ticker: &str,
        side: Side,
        price: f64,
        timestamp: i64,
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str
    ) -> Option<TradeLog> {
        let quantity = 10; // Fixed for now
        let cost = price * quantity as f64;
        
        let can_afford = match side {
             Side::Long => self.buying_power >= cost,
             Side::Short => true, 
             _ => false
        };

        if matches!(side, Side::Long) && !can_afford {
             println!("Insufficient BP for Long on {}", ticker);
             return None;
        }

        match side {
            Side::Long => self.buying_power -= cost,
            Side::Short => self.buying_power += cost,
            _ => {}
        }

        let id = Uuid::new_v4().to_string();
        let pos = Position::new(
            id,
            ticker.to_string(),
            side,
            quantity,
            price,
            timestamp
        );
        self.open_positions.insert(ticker.to_string(), pos.clone());
        
        Some(TradeLog::new(
            pos,
            Action::Entry,
            strategy_name.to_string(),
            "Signal".to_string(),
            indicator_values.clone()
        ))
    }
}
