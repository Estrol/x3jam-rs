#[repr(u16)]
#[derive(Debug, Clone, Copy, encoder::StructDeserializer, encoder::StructSerializer)]
pub enum GameEventType {
    Life = 0,
    Jam = 1,
}
