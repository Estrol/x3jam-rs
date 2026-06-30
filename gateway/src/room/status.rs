#[repr(u8)]
#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer, PartialEq, Eq)]
pub enum RoomStatus {
    Waiting = 1,
    Playing = 2,
}
