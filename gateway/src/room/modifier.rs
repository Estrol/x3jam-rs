#[derive(
    Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer, Debug, Hash, PartialEq, Eq,
)]
pub enum Modifiers {
    None = 0,
    Rate = 1,
    Timing = 2,
    Fln = 3,
    Sln = 4,
    Nln = 5,
}

#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct ModifierReport {
    pub rate: f32,
    pub timing_bpm: u32,
    pub fln: u32,
    pub sln: u32,
    pub nln: u32,
}
