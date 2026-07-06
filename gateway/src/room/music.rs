// If you using old music id system, you can enable `disable-o2hook2-mod` feature to use u16 as music id instead of u32.
// But beaware that this will limit the maximum music id to 4095, which may cause issues with some songs.
// DMJAM server has this issues already.

#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct MusicId {
    #[cfg(not(feature = "disable-o2hook2-mod"))]
    pub value: u32,
    #[cfg(feature = "disable-o2hook2-mod")]
    pub value: u16,
}

impl MusicId {
    pub fn new(songid: u32) -> Self {
        Self {
            #[cfg(not(feature = "disable-o2hook2-mod"))]
            value: songid & 0xFFFF | 0xF0000,
            #[cfg(feature = "disable-o2hook2-mod")]
            value: (songid & 0xFFF) as u16 | 0xF000,
        }
    }

    pub fn available(self) -> Self {
        Self {
            #[cfg(not(feature = "disable-o2hook2-mod"))]
            value: self.value | 0xF0000,
            #[cfg(feature = "disable-o2hook2-mod")]
            value: self.value | 0xF000,
        }
    }

    pub fn unavailable(self) -> Self {
        Self {
            #[cfg(not(feature = "disable-o2hook2-mod"))]
            value: self.value & !0xF0000,
            #[cfg(feature = "disable-o2hook2-mod")]
            value: self.value & !0xF000,
        }
    }

    pub fn is_available(self) -> bool {
        #[cfg(not(feature = "disable-o2hook2-mod"))]
        {
            (self.value & 0xF0000) != 0
        }
        #[cfg(feature = "disable-o2hook2-mod")]
        {
            (self.value & 0xF000) != 0
        }
    }

    pub fn songid(self) -> u32 {
        #[cfg(not(feature = "disable-o2hook2-mod"))]
        {
            self.value & 0xFFFF
        }
        #[cfg(feature = "disable-o2hook2-mod")]
        {
            (self.value & 0x0FFF) as u32
        }
    }
}

impl std::fmt::Display for MusicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.value);
    }
}

impl Eq for MusicId {}

impl PartialEq for MusicId {
    fn eq(&self, other: &Self) -> bool {
        self.songid() == other.songid()
    }
}
