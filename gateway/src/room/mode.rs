#[repr(u8)]
#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub enum RoomMode {
    Solo = 0,
    Versus = 1,
    Couple = 2,
    Jam = 3,
}
