#[derive(Debug, Clone)]
pub enum Side{
    Long,
    Short,
    None
}

impl Side{
    pub fn to_string(side: &Self) -> &str {
        match side{
            Side::Long => "long",
            Side::Short => "short",
            Side::None => "none"
        }
    }
}