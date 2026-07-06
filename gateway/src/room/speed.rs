#[repr(u8)]
#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub enum RoomSpeed {
    Speed10 = 0x00,
    Speed15 = 0x01,
    Speed20 = 0x02,
    Speed25 = 0x03,
    Speed30 = 0x04,
    Speed35 = 0x05,
    Speed40 = 0x06,
    Speed50 = 0x07,
    Speed60 = 0x08,
    Speed05 = 0x09,
}
