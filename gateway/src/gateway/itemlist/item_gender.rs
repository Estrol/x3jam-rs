#[derive(Debug, Clone, Copy, Default)]
pub enum ItemGender {
    #[default]
    Female = 0,
    Male = 1,
    Any = 2,
}

impl ItemGender {
    pub fn from_flags(flags: u16) -> Self {
        match (flags >> 7) & 15 {
            0 => Self::Female,
            1 => Self::Male,
            _ => Self::Any,
        }
    }
}
