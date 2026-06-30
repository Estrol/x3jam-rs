#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct MusicId(
    #[cfg(not(feature = "music-id-u32"))] pub u16,
    #[cfg(feature = "music-id-u32")] pub u32,
);

#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct MusicIdEntry {
    pub songid: MusicId,
    pub flag: u8,
}
