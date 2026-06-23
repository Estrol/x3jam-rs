#[repr(u8)]
#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub enum TeamId {
    Red = 0,
    Orange = 1,
    Yellow = 2,
    Green = 3,
    Cyan = 4,
    Blue = 5,
    Purple = 6,
    DarkRed = 7,
}
