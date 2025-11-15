/// DEFINE ENTRY CONDITIONS
/// 

/// WANT TO BE ABLE TO PASS AN ARRAY OF BOOL OUTPUT FUNCTIONS
/// SHOULD OVERLOAD THE >,< operators FOR indicators::indicator::Indicator TYPE
/// THEN PASS A LIST OF ENTRY CONDITIONS
/// 
/// i1 = MovingAverage::new(Window::Hours(3).rounded(), CommonField::Close)
/// i2 = MovingAverage::new(Window::Hours(5), CommonField::Typical)
/// i3 = ADV::new(5)
/// e1 = EntryCondition::new(i1 > i2 and i3 > 1e5, PositionTiming::MarketClose) // PositionTiming::MarketOpen, PositionTiming::NextBarOpen, PositionTiming::NextBarClose
