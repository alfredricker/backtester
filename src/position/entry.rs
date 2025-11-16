
pub struct EntryStrategy {
    /// Name of this strategy
    pub name: String,
    /// Conditions that must be met to enter
    pub conditions: Vec<Box<dyn Event>>,
    /// How to determine position size
    pub sizing: SizingStrategy,
    /// Position side (Long/Short)
    pub side: Side,
    /// Optional stop loss configuration
    pub stop_loss: Option<StopLossConfig>,
    /// Optional take profit configuration
    pub take_profit: Option<TakeProfitConfig>,
    /// Optional maximum time in position
    pub max_time: Option<i64>,
}

impl EntryStrategy {
    /// Create a new entry strategy
    pub fn new(name: String, side: PositionSide, sizing: SizingStrategy) -> Self {
        Self {
            name,
            conditions: Vec::new(),
            sizing,
            side,
            stop_loss: None,
            take_profit: None,
            max_time: None,
        }
    }
    
    /// Add an entry condition
    pub fn add_condition(&mut self, condition: Box<dyn Event>) {
        self.conditions.push(condition);
    }
    
    /// Check if all entry conditions are met
    pub fn check_entry(&mut self, indicators: &[Indicator], row: &Row) -> bool {
        if self.conditions.is_empty() {
            return false;
        }
        
        // All conditions must be true
        self.conditions.iter_mut().all(|cond| cond.check(indicators, row))
    }
    
    /// Calculate position size based on strategy
    pub fn calculate_size(&self, row: &Row, account_value: f64) -> i64 {
        self.sizing.calculate(row, account_value)
    }
}