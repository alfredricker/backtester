use crate::position::order::OrderType;

/// Represents the output of a strategy decision
/// 
/// Signals can be:
/// 1. Boolean (Trigger): Simple "Do X" because condition met
/// 2. Value (Score): "Rank is X", used for ordering/allocation
#[derive(Debug, Clone)]
pub enum SignalType {
    /// A binary decision (e.g., "Buy", "Sell")
    Trigger(OrderType),
    /// A continuous value (e.g., "Sentiment Score", "Momentum Strength")
    /// High values might imply stronger conviction or priority
    Value(f64),
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub ticker: String,
    pub signal_type: SignalType,
    pub reason: String,
}

impl Signal {
    pub fn new_trigger(ticker: String, order_type: OrderType, reason: String) -> Self {
        Self {
            ticker,
            signal_type: SignalType::Trigger(order_type),
            reason,
        }
    }

    pub fn new_value(ticker: String, value: f64, reason: String) -> Self {
        Self {
            ticker,
            signal_type: SignalType::Value(value),
            reason,
        }
    }
}
