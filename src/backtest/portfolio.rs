use std::collections::HashMap;
use crate::{config, position::side::Side};
use super::signal::Signal;
use crate::position::order::OrderType;

#[derive(Debug, Clone)]
pub struct Position {
    pub ticker: String,
    pub side: Side,
    pub quantity: i64,
    pub average_entry_price: f64,
    pub current_price: f64,
}

impl Position {
    pub fn pnl(&self) -> f64 {
        let c: f64;
        match self.side {
            Side::Long => c = 1.,
            Side::Short => c = -1.,
            Side::None => c = 0.
        };
        c*(self.quantity as f64)*(self.average_entry_price - self.current_price)
    }
}

// instead of Position could do a "Status" struct with Deque<Order> for pending orders,

pub struct Portfolio {
    pub buying_power: f64,
    pub positions: HashMap<String, Position>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            buying_power: config::get_config().starting_buying_power,
            positions: HashMap::new(),
        }
    }

    pub fn update_prices(&mut self, ticker: &str, price: f64) {
        if let Some(pos) = self.positions.get_mut(ticker) {
            pos.current_price = price;
        }
    }

    pub fn process_signal(&mut self, signal: &Signal, price: f64) {
        use super::signal::SignalType;
        
        match &signal.signal_type {
            SignalType::Trigger(order_type) => {
                match order_type {
                    OrderType::MarketBuy() => self.execute_buy(&signal.ticker, *qty, price),
                    OrderType::MarketSell() => self.execute_sell(&signal.ticker, *qty, price),
                    _ => println!("Unsupported order type for simple backtest: {:?}", order_type),
                }
            },
            SignalType::Value(val) => {
                println!("Received value signal for {}: {:.4}. Portfolio weighting not yet implemented.", signal.ticker, val);
            }
        }
    }

    pub fn submit_order(&mut self, order: &Order){}

    fn check_orders(&mut self, ticker: &str, row: &Row) {
        let cost = quantity as f64 * price;
        // Simple slippage model: add 0.1% to price
        let slippage = cost * 0.001; 
        let total_cost = cost + slippage;

        if total_cost > self.buying_power {
            println!("Insufficient buying power for {} shares of {} at {}", quantity, ticker, price);
            return;
        }

        self.buying_power -= total_cost;

        let position = self.positions.entry(ticker.to_string()).or_insert(Position {
            ticker: ticker.to_string(),
            quantity: 0,
            average_entry_price: 0.0,
            current_price: price,
        });

        // Update average entry price
        let old_cost = position.quantity as f64 * position.average_entry_price;
        let new_cost = old_cost + cost;
        position.quantity += quantity;
        position.average_entry_price = new_cost / position.quantity as f64;
        position.current_price = price;
        
        println!("BOUGHT {} x {} @ {:.2} (Total: {:.2})", quantity, ticker, price, total_cost);
    }

    fn execute_sell(&mut self, ticker: &str, quantity: i64, price: f64) {
        if let Some(position) = self.positions.get_mut(ticker) {
            if position.quantity < quantity {
                println!("Cannot sell {} shares of {}: only have {}", quantity, ticker, position.quantity);
                return;
            }

            let proceed = quantity as f64 * price;
            let slippage = proceed * 0.0001;
            let net_proceed = proceed - slippage;

            self.buying_power += net_proceed;
            position.quantity -= quantity;
            
            // Calculate realized PnL for logging (simplified)
            let entry_val = quantity as f64 * position.average_entry_price;
            let pnl = net_proceed - entry_val;
            
            println!("SOLD {} x {} @ {:.2} (PnL: {:.2})", quantity, ticker, price, pnl);

            if position.quantity == 0 {
                self.positions.remove(ticker);
            }
        }
    }
}

