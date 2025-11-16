


// could be profit target, time based, stop loss, etc.
pub struct ExitStrategy {
    pub condition: Option<Condition>,
    pub size: SizingStrategy,
    pub order: OrderType,
}

pub struct Entry {
    pub condition: Condition,
    pub order: OrderType, // order gives you side and size
    pub size: SizingStrategy,
    pub exit: Vec<ExitStrategy>, // can have multiple exit strategies
}

// want <Action, Condition> vec?
pub struct Strategy {
    pub name: String,
    pub _condition: Condition, // can be built from multiple conditions
    pub sizing: SizingStrategy,
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