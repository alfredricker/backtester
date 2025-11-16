use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::indicators::fields::CommonField;
use std::ops::{BitAnd, BitOr, Not};

// THIS FILE WOULD BE CLEANER IF WE DIDN'T HAVE TO PASS AN OPTION<ROW> OR ROW TO EVERYTHING --- BUT I WANT COMPATIBILITY WITH COMMONFIELD
pub trait Conditionable {
    fn evaluate(&self, row: Option<&Row>) -> Option<f64>;
}

impl Conditionable for f64 {
    fn evaluate(&self, _row: Option<&Row>) -> Option<f64> {
        Some(*self)
    }
}

impl Conditionable for CommonField {
    fn evaluate(&self, row: Option<&Row>) -> Option<f64> {
        row.map(|r| self.extract(r))
    }
}

impl Conditionable for Box<dyn Indicator> {
    fn evaluate(&self, _row: Option<&Row>) -> Option<f64> {
        self.get()
    }
}


/// A general condition that can be evaluated. Types possible for the generics are f64, CommonField
#[derive(Debug, Clone)]
struct Condition<L: Conditionable, R: Conditionable> {
    left: L,
    right: R
}

impl<L: Conditionable, R: Conditionable> Condition<L,R>{
    pub fn ge(&self, row: &Row) -> bool {
        match (self.left.evaluate(Some(row)), self.right.evaluate(Some(row))) {
            (Some(l), Some(r)) => l >= r,
            _ => false,  // If either side can't be evaluated, condition is false
        }
    }
    
    pub fn gt(&self, row: &Row) -> bool {
        match (self.left.evaluate(Some(row)), self.right.evaluate(Some(row))) {
            (Some(l), Some(r)) => l > r,
            _ => false,
        }
    }
    
    pub fn lt(&self, row: &Row) -> bool {
        match (self.left.evaluate(Some(row)), self.right.evaluate(Some(row))) {
            (Some(l), Some(r)) => l < r,
            _ => false,
        }
    }
    
    pub fn le(&self, row: &Row) -> bool {
        match (self.left.evaluate(Some(row)), self.right.evaluate(Some(row))) {
            (Some(l), Some(r)) => l <= r,
            _ => false,
        }
    }
}