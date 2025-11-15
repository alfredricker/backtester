
pub struct EntryCondition{
    // how do we handle condition creation?
        
}

pub struct ExitCondition{

}

// assumes market exit -- unrealistic for larger size -- need a liquidity limit order model
pub enum PositionTiming{
    MarketOpen,
    MarketClose,
    NextBarOpen,
    NextBarClose
}

pub struct PositionCondition{
    pub entry_condition: EntryCondition,
    pub exit_condition: Option<ExitCondition>,
}