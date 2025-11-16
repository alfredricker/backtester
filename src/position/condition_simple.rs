// Example usage with the new operator overloading syntax:
//
// let condition_a = Value::Indicator(0).gt(Value::Constant(100.0)) 
//                 & Value::Indicator(1).lt(Value::Constant(200.0));
//
// let condition_b = Value::Indicator(2).eq(Value::Indicator(1)) 
//                 | Value::Indicator(3).lt(Value::Constant(400.0));
//
// let condition_c = condition_a & condition_b;
//
// You can also use negation:
// let condition_d = !Value::Indicator(0).gt(Value::Constant(100.0));
//
// Note: Use & and | (not && and ||) because Rust doesn't allow 
// overloading logical operators to return custom types.
