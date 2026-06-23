#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameModifier {
    #[default]
    None = 0,
    Power = 1,
    Mirror = 2,
    Random = 3,
    Panic = 4,
    Hidden = 5,
    Sudden = 6,
    Dark = 7,
}

impl GameModifier {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Power,
            2 => Self::Mirror,
            3 => Self::Random,
            4 => Self::Panic,
            5 => Self::Hidden,
            6 => Self::Sudden,
            7 => Self::Dark,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameModifierType {
    #[default]
    None = 0,
    Power = 1,
    Arrange = 2,
    Visiblity = 3,
}

impl GameModifierType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Power,
            2 => Self::Arrange,
            3 => Self::Visiblity,
            _ => Self::None,
        }
    }
}
