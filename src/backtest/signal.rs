use crate::position::order::OrderType;

#[derive(Debug, Clone)]
pub enum SignalType {
    /// A binary decision (e.g., "Buy", "Sell")
    Trigger(OrderType),
    /// A continuous value (e.g., "Sentiment Score", "Momentum Strength")
    /// High values might imply stronger conviction or priority
    Value(f64),
}

/// Represents the output of a strategy decision
#[derive(Debug, Clone)]
pub struct Signal {
    pub ticker: String,
    pub signal_type: SignalType,
}

impl Signal {
    pub fn new_trigger(ticker: String, order_type: OrderType) -> Self {
        Self {
            ticker,
            signal_type: SignalType::Trigger(order_type),
        }
    }

    pub fn new_value(ticker: String, value: f64) -> Self {
        Self {
            ticker,
            signal_type: SignalType::Value(value),
        }
    }
}
