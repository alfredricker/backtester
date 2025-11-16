use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::indicators::fields::CommonField;

// Example usage to create: i1 > i2 and i3 > 1e5
// 
// let condition = Condition::and(vec![
//     Condition::gt(Value::Indicator(0), Value::Indicator(1)),  // i1 > i2
//     Condition::gt(Value::Indicator(2), Value::Constant(1e5)), // i3 > 1e5
// ]);

/// Represents a value that can be compared in a condition
#[derive(Debug, Clone)]
pub enum Value {
    /// Reference to an indicator by index in the indicator array
    Indicator(usize),
    /// A constant value
    Constant(f64),
    /// A field from the current row (e.g., close, volume)
    Field(CommonField),
}

impl Value {
    /// Evaluate this value given the current context
    pub fn evaluate(&self, indicators: &[Indicator], row: &Row) -> Option<f64> {
        match self {
            Value::Indicator(idx) => indicators.get(*idx).and_then(|ind| ind.get()),
            Value::Constant(val) => Some(*val),
            Value::Field(field) => Some(field.extract(row)),
        }
    }
}

/// Comparison operators
#[derive(Debug, Clone, Copy)]
pub enum Comparison {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

impl Comparison {
    pub fn evaluate(&self, left: f64, right: f64) -> bool {
        match self {
            Comparison::GreaterThan => left > right,
            Comparison::GreaterThanOrEqual => left >= right,
            Comparison::LessThan => left < right,
            Comparison::LessThanOrEqual => left <= right,
            Comparison::Equal => (left - right).abs() < f64::EPSILON,
            Comparison::NotEqual => (left - right).abs() >= f64::EPSILON,
        }
    }
}

/// A general condition that can be evaluated
#[derive(Debug, Clone)]
pub enum Condition {
    /// Compare two values
    Compare {
        left: Value,
        op: Comparison,
        right: Value,
    },
    /// Logical AND of multiple conditions
    And(Vec<Condition>),
    /// Logical OR of multiple conditions
    Or(Vec<Condition>),
    /// Logical NOT
    Not(Box<Condition>),
}

impl Condition {
    /// Evaluate this condition given the current context
    pub fn evaluate(&self, indicators: &[Indicator], row: &Row) -> bool {
        match self {
            Condition::Compare { left, op, right } => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    op.evaluate(l, r)
                } else {
                    false // If we can't evaluate, condition is false
                }
            }
            Condition::And(conditions) => {
                conditions.iter().all(|c| c.evaluate(indicators, row))
            }
            Condition::Or(conditions) => {
                conditions.iter().any(|c| c.evaluate(indicators, row))
            }
            Condition::Not(condition) => {
                !condition.evaluate(indicators, row)
            }
        }
    }
    
    /// Builder: Create a greater-than comparison
    pub fn gt(left: Value, right: Value) -> Self {
        Condition::Compare { 
            left, 
            op: Comparison::GreaterThan, 
            right 
        }
    }
    
    /// Builder: Create a less-than comparison
    pub fn lt(left: Value, right: Value) -> Self {
        Condition::Compare { 
            left, 
            op: Comparison::LessThan, 
            right 
        }
    }
    
    /// Builder: Combine multiple conditions with AND
    pub fn and(conditions: Vec<Condition>) -> Self {
        Condition::And(conditions)
    }
    
    /// Builder: Combine multiple conditions with OR
    pub fn or(conditions: Vec<Condition>) -> Self {
        Condition::Or(conditions)
    }
}

