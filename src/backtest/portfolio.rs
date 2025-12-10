use std::collections::HashMap;
use std::collections::VecDeque;
use crate::config::{self, ReplacementStrategy};
use crate::position::side::Side;
use crate::position::position::Position;
use crate::types::log::TradeLog;
use crate::backtest::signal::{Signal, SignalType};
use crate::position::order::{Order, OrderType, OrderAction};
use crate::position::strategy::Action;
use crate::types::ohlcv::Row;
use uuid::Uuid;

pub struct PendingOrder {
    pub order: Order,
    pub ticker: String,
    pub strategy_name: String,
    pub indicator_values: HashMap<String, f64>,
}

pub struct Portfolio {
    pub buying_power: f64,
    pub open_positions: HashMap<String, Position>, // Ticker -> Position
    pub closed_positions: Vec<Position>,
    pub pending_orders: VecDeque<PendingOrder>, // FIFO queue for pending orders
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            buying_power: config::get_config().starting_buying_power,
            open_positions: HashMap::new(),
            closed_positions: Vec::new(),
            pending_orders: VecDeque::new(),
        }
    }

    pub fn update_prices(&mut self, _ticker: &str, _price: f64) {
        // In this simple model, we don't store current price in Position struct persistently.
        // We could track unrealized PnL here if needed.
    }

    /// Process a new signal, potentially creating a pending order
    pub fn process_signal(
        &mut self, 
        signal: &Signal, 
        price: f64, 
        timestamp: i64, 
        indicator_values: &HashMap<String, f64>,
        strategy_name: &str,
    ) {
        // Determine Action and create Order
        match &signal.signal_type {
            SignalType::Trigger(order_type) => {
                let is_buy = order_type.is_buy();
                let is_sell = order_type.is_sell();

                // Determine OrderAction based on current position state
                // Simplification: 
                // - If we have a position and receive opposite signal -> Close
                // - If we have no position and receive entry signal -> Open
                // - If we have position and receive same signal -> Ignore (or add size, but let's stick to 1 pos per ticker)
                
                let maybe_pos = self.open_positions.get(&signal.ticker);
                let (action, side) = match maybe_pos {
                    Some(pos) => {
                        if (is_buy && matches!(pos.side, Side::Short)) || (is_sell && matches!(pos.side, Side::Long)) {
                            (OrderAction::Close, pos.side.clone()) // Closing the existing side
                        } else {
                            return; // Signal matches current position or invalid
                        }
                    },
                    None => {
                        if is_buy {
                            (OrderAction::Open, Side::Long)
                        } else if is_sell {
                            (OrderAction::Open, Side::Short)
                        } else {
                            return;
                        }
                    }
                };

                // Create the Order object
                let size = config::get_config().sizing_strategy.calculate(price, self.buying_power, Some(signal));
                // If closing, use position size
                let order_size = if let OrderAction::Close = action {
                    if let Some(pos) = maybe_pos { pos.size } else { size }
                } else {
                    size
                };

                let order_res = Order::new(
                    *order_type,
                    action.clone(),
                    timestamp,
                    None, // Default good_until
                    order_size
                );

                if let Ok(order) = order_res {
                    // Check replacement strategy for NEW OPEN orders
                    if let OrderAction::Open = action {
                         // Estimate cost (Market orders use current price)
                         // Limit orders we might use limit price
                         // For now use current price as estimate
                         let estimated_cost = price * order_size as f64;
                         
                         if estimated_cost > self.buying_power {
                            self.handle_replacement_strategy(
                                PendingOrder {
                                    order,
                                    ticker: signal.ticker.clone(),
                                    strategy_name: strategy_name.to_string(),
                                    indicator_values: indicator_values.clone(),
                                }
                            );
                            return;
                         }
                    }
                    
                    // Add to pending
                    self.pending_orders.push_back(PendingOrder {
                        order,
                        ticker: signal.ticker.clone(),
                        strategy_name: strategy_name.to_string(),
                        indicator_values: indicator_values.clone(),
                    });
                }
            },
            SignalType::Value(_) => {} // Ignore value signals for now
        }
    }

    fn handle_replacement_strategy(&mut self, pending: PendingOrder) {
        let strategy = config::get_config().replacement_strategy;
        match strategy {
            ReplacementStrategy::Cancel => {
                // Drop the order
                println!("Insufficient BP for {}. Order Cancelled.", pending.ticker);
            },
            ReplacementStrategy::Queue => {
                // Add to queue anyway. It will be checked in check_orders each tick
                println!("Insufficient BP for {}. Order Queued.", pending.ticker);
                self.pending_orders.push_back(pending);
            },
            ReplacementStrategy::ReplaceOldest => {
                // Find oldest open position
                // We need to look at self.open_positions
                if let Some(oldest_ticker) = self.get_oldest_position_ticker() {
                    // Issue a CLOSE order for oldest
                    // Then queue the new order
                    // Note: This relies on next tick to process the close, freeing BP.
                    // The new order might still fail BP check this tick if added after.
                    // Ideally we'd execute close immediately, but we stick to order flow.
                    // We'll queue the close order at front?
                    
                    if let Some(pos) = self.open_positions.get(&oldest_ticker) {
                        // Create close order
                        let close_type = match pos.side {
                            Side::Long => OrderType::MarketSell(),
                            Side::Short => OrderType::MarketBuy(),
                            Side::None => return,
                        };
                        
                        if let Ok(close_order) = Order::new(
                             close_type,
                             OrderAction::Close,
                             pending.order.timestamp, // use same timestamp
                             None,
                             pos.size
                        ) {
                            // Push close order to front to be processed first
                            self.pending_orders.push_front(PendingOrder {
                                order: close_order,
                                ticker: oldest_ticker.clone(),
                                strategy_name: "Replacement".to_string(),
                                indicator_values: HashMap::new(),
                            });
                            
                            // Queue new order at back
                            println!("ReplaceOldest triggered: Closing {} for {}", oldest_ticker, pending.ticker);
                            self.pending_orders.push_back(pending);
                        }
                    }
                } else {
                     // No positions to replace
                     println!("Insufficient BP for {}. No positions to replace.", pending.ticker);
                }
            },
             ReplacementStrategy::ReplaceNewest => {
                if let Some(newest_ticker) = self.get_newest_position_ticker() {
                     if let Some(pos) = self.open_positions.get(&newest_ticker) {
                        let close_type = match pos.side {
                            Side::Long => OrderType::MarketSell(),
                            Side::Short => OrderType::MarketBuy(),
                            Side::None => return,
                        };
                         if let Ok(close_order) = Order::new(
                             close_type,
                             OrderAction::Close,
                             pending.order.timestamp,
                             None,
                             pos.size
                        ) {
                             self.pending_orders.push_front(PendingOrder {
                                order: close_order,
                                ticker: newest_ticker.clone(),
                                strategy_name: "Replacement".to_string(),
                                indicator_values: HashMap::new(),
                            });
                            println!("ReplaceNewest triggered: Closing {} for {}", newest_ticker, pending.ticker);
                            self.pending_orders.push_back(pending);
                        }
                     }
                }
            },
            ReplacementStrategy::ReplaceSignal => {
                 // Needs signal comparison logic. For now, behave like Cancel or Queue
                 println!("ReplaceSignal not fully implemented. Queuing.");
                 self.pending_orders.push_back(pending);
            }
        }
    }

    fn get_oldest_position_ticker(&self) -> Option<String> {
        self.open_positions.values()
            .min_by_key(|p| p.entry_timestamp)
            .map(|p| p.ticker.clone())
    }

    fn get_newest_position_ticker(&self) -> Option<String> {
        self.open_positions.values()
            .max_by_key(|p| p.entry_timestamp)
            .map(|p| p.ticker.clone())
    }

    /// Check all pending orders against current market data
    /// Returns any generated TradeLogs
    pub fn check_orders(&mut self, row: &Row) -> Vec<TradeLog> {
        let mut logs = Vec::new();
        let mut remaining_orders = VecDeque::new();
        
        // We only process orders for the current ticker in the row?
        // No, pending_orders queue might contain orders for other tickers.
        // We should only check orders matching the current row's ticker.
        // But popping from VecDeque efficiently is hard if we skip.
        // Strategy: Iterate whole queue. If ticker matches, check. If kept, push to new queue.
        // If ticker doesn't match, push to new queue.
        
        // BETTER: Store pending_orders as HashMap<Ticker, VecDeque<PendingOrder>>
        // But for ReplacementStrategy queue (global BP), a global queue makes sense for priority.
        // But we can only fill orders if we have data (row).
        // Let's assume we iterate all, but only "check" matching ticker.
        
        // Optimization: Use separate queues or index?
        // For now, simple iteration.
        
        while let Some(mut pending) = self.pending_orders.pop_front() {
            if pending.ticker != row.ticker {
                remaining_orders.push_back(pending);
                continue;
            }

            // Check order
            if let Ok(_) = pending.order.check(row) {
                if pending.order.completed {
                     // Order Filled
                     if let Some(fill_price) = pending.order.fill_price {
                         // Execute Trade
                         if let Some(log) = self.execute_trade(pending, fill_price) {
                             logs.push(log);
                         } else {
                             // Execution failed (e.g. BP check for Open order in Queue)
                             // If it was Open and failed BP, maybe keep in queue?
                             // But check() already marked it completed. We'd need to reset or recreate.
                             // For simplicity: If execution fails due to BP, we drop it (or log error).
                         }
                     }
                } else {
                    // Not filled, but still active
                    remaining_orders.push_back(pending);
                }
            } else {
                 // Error checking order (e.g. expired?)
                 if pending.order.completed {
                     // Expired or cancelled
                 } else {
                     remaining_orders.push_back(pending);
                 }
            }
        }
        
        self.pending_orders = remaining_orders;
        logs
    }

    fn execute_trade(&mut self, pending: PendingOrder, fill_price: f64) -> Option<TradeLog> {
         let size = pending.order.fill_size; // should use fill_size
         
         match pending.order.open_or_close {
             OrderAction::Open => {
                 let cost = fill_price * size as f64;
                 if self.buying_power < cost {
                     println!("Order filled but insufficient BP for execution: {}", pending.ticker);
                     return None;
                 }
                 
                 self.buying_power -= cost;
                 let id = Uuid::new_v4().to_string();
                 let pos = Position::new(
                     id,
                     pending.ticker.clone(),
                     // Infer side from OrderType. 
                     // MarketBuy -> Long, MarketSell -> Short (for Open)
                     if pending.order.order_type.is_buy() { Side::Long } else { Side::Short },
                     size,
                     fill_price,
                     pending.order.timestamp
                 );
                 
                 self.open_positions.insert(pending.ticker.clone(), pos.clone());
                 
                 Some(TradeLog::new(
                     pos,
                     Action::Entry,
                     pending.strategy_name,
                     "OrderFilled".to_string(),
                     pending.indicator_values
                 ))
             },
             OrderAction::Close => {
                 if let Some(mut pos) = self.open_positions.remove(&pending.ticker) {
                     // Force close logic on position struct
                     if let Ok(_) = pos.close(fill_price, pending.order.timestamp) {
                          // Update BP
                        match pos.side {
                            Side::Short => {
                                let cost = fill_price * pos.size as f64;
                                self.buying_power -= cost; // Short exit you pay back
                                // Wait, simple cash model:
                                // Short Open: BP += Proceeds. 
                                // Short Close: BP -= Cost.
                                // Correct.
                            }
                            Side::Long => {
                                let proceeds = fill_price * pos.size as f64;
                                self.buying_power += proceeds;
                            }
                            _ => {}
                        }
                        
                        let log = TradeLog::new(
                             pos.clone(),
                             Action::Exit,
                             pending.strategy_name,
                             "OrderFilled".to_string(),
                             pending.indicator_values
                         );
                         self.closed_positions.push(pos);
                         Some(log)
                     } else {
                         None
                     }
                 } else {
                     None
                 }
             }
         }
    }
}
