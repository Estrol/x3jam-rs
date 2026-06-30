#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer, PartialEq, Eq)]
pub struct MusicId(
    pub u16
);

impl std::fmt::Display for MusicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct MusicIdEntry {
    pub songid: MusicId,
    #[cfg(feature = "separate-music-id-flag")]
    pub flag: u8,
}

impl std::fmt::Display for MusicIdEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(feature = "separate-music-id-flag"))]
        return write!(f, "{}", self.songid);

        #[cfg(feature = "separate-music-id-flag")]
        return write!(f, "{}:{}", self.songid, self.flag);
    }
}