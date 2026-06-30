#[repr(u8)]
#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub enum RoomDifficulty {
    Easy = 0,
    Normal = 1,
    Hard = 2,
}
