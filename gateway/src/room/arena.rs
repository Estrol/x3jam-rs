bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct RoomArena: u32 {
        const ARENA1 = 0x1;
        const ARENA2 = 0x2;
        const ARENA3 = 0x3;
        const ARENA4 = 0x4;
        const ARENA5 = 0x5;
        const ARENA6 = 0x6;
        const ARENA7 = 0x7;
        const ARENA8 = 0x8;
        const ARENA9 = 0x9;
        const ARENA10 = 0xA;
        const ARENA11 = 0xB;
        const ARENA12 = 0xC;
        const ARENA13 = 0xD;

        const RANDOM_FLAG = 0x80 << 24; // Random flag in the upper bits
    }
}

impl encoder::StructDecodeImpl for RoomArena {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        let value = u32::impl_decode(reader)?;
        Ok(Self::from_bits_truncate(value))
    }
}

impl encoder::StructEncodeImpl for RoomArena {
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.bits().impl_encode(writer)
    }
}
