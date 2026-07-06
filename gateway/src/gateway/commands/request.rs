use byteorder_lite::{LittleEndian, ReadBytesExt as _};

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RequestId {
    Disconnect = 0xFFF0, // Done

    GatewayConnect = 0x03F1,   // Done
    GatewayLogin = 0x03EF,     // Done
    GatewayPing = 0x1771,      // Done
    GatewayReconnect = 0x03F3, // Done
    GatewayReauth = 0x03E8,    // Done

    PlanetGetChannels = 0x03EA,   // Done
    PlanetGetServerList = 0x0FBE, // Done
    PlanetEnterChannel = 0x03EC,  // Done
    PlanetLeaveChannel = 0x07E5,  // Done

    ListRoomGetCharacter = 0x07D0,  // Done
    ListRoomGetRoomList = 0x07D2,   // Done
    ListRoomSyncInfo = 0x13A4,      // Done
    ListRoomGetClientList = 0x07E8, // Done
    ListRoomCreateRoom = 0x07D4,    // Done
    ListRoomLeaveRoom = 0x0BBD,     // Done
    ListRoomJoinRoom = 0x0BBA,      // Done
    ListRoomChat = 0x07DC,

    ShopEnter = 0x138F,
    ShopLeave = 0x138E,
    ShopActionBuy = 0x1397,
    ShopActionSync = 0x1388,

    EquipItem = 0x138C,

    RoomSetArena = 0x0FA2,      // Done
    RoomSetMusicId = 0x0FA0,    // Done
    RoomSetSkill = 0x0FB7,      // Done
    RoomSetReady = 0x0FA8,      // Done
    RoomSetTeam = 0x0FA4,       // Done
    RoomChat = 0x0BC3,          // Done
    RoomNameChange = 0x0BB8,    // Done
    RoomSlotToggle = 0x0BC0,    // Done
    RoomSetMusicState = 0x0FB9, // Done

    GameStart = 0x0FAA,
    GameLeave = 0x0FB5,
    GameConfirmLoaded = 0x0FAC,
    GameNoteEvent = 0x0FAE,
    SubmitScore = 0x0FB0,

    // O2Hook2's Extensions
    #[cfg(not(feature = "disable-o2hook2-mod"))]
    RequestVersion = 0xAAAB,
    #[cfg(not(feature = "disable-o2hook2-mod"))]
    RoomSetModifier = 0xAAB0,
    #[cfg(not(feature = "disable-o2hook2-mod"))]
    RoomSetAllModifiers = 0xAAB1,

    Unknown(u16),
}

impl RequestId {
    pub fn from_bytes<T>(reader: &mut T) -> Self
    where
        T: std::io::Read,
    {
        match reader.read_u16::<LittleEndian>() {
            Ok(id) => match id {
                0xFFF0 => RequestId::Disconnect,
                0x03F1 => RequestId::GatewayConnect,
                0x03EF => RequestId::GatewayLogin,
                0x03F3 => RequestId::GatewayReconnect,
                0x03E8 => RequestId::GatewayReauth,
                0x03EA => RequestId::PlanetGetChannels,
                0x0FBE => RequestId::PlanetGetServerList,
                0x03EC => RequestId::PlanetEnterChannel,
                0x07E5 => RequestId::PlanetLeaveChannel,
                0x07D2 => RequestId::ListRoomGetRoomList,
                0x07D0 => RequestId::ListRoomGetCharacter,
                0x13A4 => RequestId::ListRoomSyncInfo,
                0x07E8 => RequestId::ListRoomGetClientList,
                0x1771 => RequestId::GatewayPing,
                0x07D4 => RequestId::ListRoomCreateRoom,
                0x0BBD => RequestId::ListRoomLeaveRoom,
                0x0FA2 => RequestId::RoomSetArena,
                0x0FA0 => RequestId::RoomSetMusicId,
                0x0BBA => RequestId::ListRoomJoinRoom,
                0x0FB7 => RequestId::RoomSetSkill,
                0x138E => RequestId::ShopLeave,
                0x138F => RequestId::ShopEnter,
                0x0FAA => RequestId::GameStart,
                0x0FAC => RequestId::GameConfirmLoaded,
                0x0FB5 => RequestId::GameLeave,
                0x0FAE => RequestId::GameNoteEvent,
                0x0FA8 => RequestId::RoomSetReady,
                0x0FB0 => RequestId::SubmitScore,
                0x0FA4 => RequestId::RoomSetTeam,
                0x07DC => RequestId::ListRoomChat,
                0x0BC3 => RequestId::RoomChat,
                0x0BB8 => RequestId::RoomNameChange,
                0x0BC0 => RequestId::RoomSlotToggle,
                0x1397 => RequestId::ShopActionBuy,
                0x1388 => RequestId::ShopActionSync,
                0x138C => RequestId::EquipItem,
                #[cfg(not(feature = "disable-o2hook2-mod"))]
                0xAAAB => RequestId::RequestVersion,
                #[cfg(not(feature = "disable-o2hook2-mod"))]
                0xAAB0 => RequestId::RoomSetModifier,
                #[cfg(not(feature = "disable-o2hook2-mod"))]
                0xAAB1 => RequestId::RoomSetAllModifiers,
                _ => RequestId::Unknown(id),
            },
            Err(_) => RequestId::Unknown(0),
        }
    }
}
