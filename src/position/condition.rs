use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::indicators::fields::CommonField;
use std::ops::{BitAnd, BitOr, Not};

// Example usage with operator overloading:
// 
// let condition = Value::Indicator(0).gt(Value::Constant(100.0)) 
//               & Value::Indicator(1).lt(Value::Constant(200.0));
//
// or more complex:
// let condition_a = Value::Indicator(0).gt(Value::Constant(100.0)) 
//                 & Value::Indicator(1).lt(Value::Constant(200.0));
// let condition_b = Value::Indicator(2).eq(Value::Indicator(1)) 
//                 | Value::Indicator(3).lt(Value::Constant(400.0));
// let condition_c = condition_a & condition_b;

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

    /// Greater than comparison
    pub fn gt(self, other: Value) -> Condition {
        Condition::GreaterThan(self, other)
    }

    /// Greater than or equal comparison
    pub fn gte(self, other: Value) -> Condition {
        Condition::GreaterThanOrEqual(self, other)
    }

    /// Less than comparison
    pub fn lt(self, other: Value) -> Condition {
        Condition::LessThan(self, other)
    }

    /// Less than or equal comparison
    pub fn lte(self, other: Value) -> Condition {
        Condition::LessThanOrEqual(self, other)
    }

    /// Equal comparison
    pub fn eq(self, other: Value) -> Condition {
        Condition::Equal(self, other)
    }

    /// Not equal comparison
    pub fn ne(self, other: Value) -> Condition {
        Condition::NotEqual(self, other)
    }
}

/// A general condition that can be evaluated
#[derive(Debug, Clone)]
pub enum Condition {
    /// Greater than comparison
    GreaterThan(Value, Value),
    /// Greater than or equal comparison
    GreaterThanOrEqual(Value, Value),
    /// Less than comparison
    LessThan(Value, Value),
    /// Less than or equal comparison
    LessThanOrEqual(Value, Value),
    /// Equal comparison
    Equal(Value, Value),
    /// Not equal comparison
    NotEqual(Value, Value),
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
            Condition::GreaterThan(left, right) => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    l > r
                } else {
                    false
                }
            }
            Condition::GreaterThanOrEqual(left, right) => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    l >= r
                } else {
                    false
                }
            }
            Condition::LessThan(left, right) => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    l < r
                } else {
                    false
                }
            }
            Condition::LessThanOrEqual(left, right) => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    l <= r
                } else {
                    false
                }
            }
            Condition::Equal(left, right) => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    (l - r).abs() < f64::EPSILON
                } else {
                    false
                }
            }
            Condition::NotEqual(left, right) => {
                if let (Some(l), Some(r)) = (left.evaluate(indicators, row), right.evaluate(indicators, row)) {
                    (l - r).abs() >= f64::EPSILON
                } else {
                    false
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
}

// Implement & operator for AND
impl BitAnd for Condition {
    type Output = Condition;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Flatten nested ANDs
            (Condition::And(mut left), Condition::And(right)) => {
                left.extend(right);
                Condition::And(left)
            }
            (Condition::And(mut conditions), other) | (other, Condition::And(mut conditions)) => {
                conditions.push(other);
                Condition::And(conditions)
            }
            (left, right) => Condition::And(vec![left, right]),
        }
    }
}

// Implement | operator for OR
impl BitOr for Condition {
    type Output = Condition;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Flatten nested ORs
            (Condition::Or(mut left), Condition::Or(right)) => {
                left.extend(right);
                Condition::Or(left)
            }
            (Condition::Or(mut conditions), other) | (other, Condition::Or(mut conditions)) => {
                conditions.push(other);
                Condition::Or(conditions)
            }
            (left, right) => Condition::Or(vec![left, right]),
        }
    }
}

// Implement ! operator for NOT
impl Not for Condition {
    type Output = Condition;

    fn not(self) -> Self::Output {
        Condition::Not(Box::new(self))
    }
}

