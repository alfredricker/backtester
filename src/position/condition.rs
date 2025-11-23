use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::indicators::fields::CommonField;

// THIS FILE WOULD BE CLEANER IF WE DIDN'T HAVE TO PASS AN OPTION<ROW> OR ROW TO EVERYTHING --- BUT I WANT COMPATIBILITY WITH COMMONFIELD
// trait to allow for conditions to be evaluated on any type that implements the evaluate method
pub trait Conditionable {
    fn evaluate(&self, row: Option<&Row>) -> Option<f64>;
    fn update(&mut self, _row: &Row) {} // Default implementation does nothing
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
    
    fn update(&mut self, row: &Row) {
        self.as_mut().update(row);
    }
}


/// A general condition that can be evaluated. Types possible for the generics are f64, CommonField
#[derive(Debug, Clone)]
pub struct Condition<L: Conditionable, R: Conditionable> {
    left: L,
    left_val_prev: Option<f64>,
    right: R,
    right_val_prev: Option<f64>
}

impl<L: Conditionable, R: Conditionable> Condition<L,R>{
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            left_val_prev: None,
            right,
            right_val_prev: None
        }
    }

    pub fn update(&mut self, row: &Row) {
        // Update the components (e.g. if they are indicators)
        self.left.update(row);
        self.right.update(row);
    }

    pub fn cross_above(&mut self, row: &Row) -> bool {
        let l_curr = self.left.evaluate(Some(row));
        let r_curr = self.right.evaluate(Some(row));
        
        let res = match (l_curr, r_curr, self.left_val_prev, self.right_val_prev) {
            (Some(lc), Some(rc), Some(lp), Some(rp)) => {
                 // Cross above: Left was <= Right, now Left > Right
                 lc > rc && lp <= rp
            },
            _ => false
        };
        
        // Store current as previous for next time
        self.left_val_prev = l_curr;
        self.right_val_prev = r_curr;
        
        res
    }

    pub fn cross_below(&mut self, row: &Row) -> bool {
        let l_curr = self.left.evaluate(Some(row));
        let r_curr = self.right.evaluate(Some(row));
        
        let res = match (l_curr, r_curr, self.left_val_prev, self.right_val_prev) {
            (Some(lc), Some(rc), Some(lp), Some(rp)) => {
                 // Cross below: Left was >= Right, now Left < Right
                 lc < rc && lp >= rp
            },
            _ => false
        };
        
        // Store current as previous for next time
        self.left_val_prev = l_curr;
        self.right_val_prev = r_curr;
        
        res
    }
}