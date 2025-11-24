use crate::position::order::OrderType;

/// Represents the output of a strategy decision
/// 
/// Signals can be:
/// 1. Boolean (Trigger): Simple "Do X" because condition met
/// 2. Value (Score): "Rank is X", used for ordering/allocation
#[derive(Debug, Clone)]
pub enum Signal {
    /// A binary decision (e.g., "Buy", "Sell")
    Trigger(OrderType),
    /// A continuous value (e.g., "Sentiment Score", "Momentum Strength")
    /// High values might imply stronger conviction or priority
    Value(f64),
}