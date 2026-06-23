#[repr(u16)]
#[derive(Debug, Copy, Clone, encoder::StructSerializer, PartialEq, Eq)]
pub enum ResponseId {
    // Gateway
    GatewayConnect = 0x03F2,
    GatewayAuth = 0x03F0,
    GatewayPing = 0x1772,
    GatewayReconnect = 0x03F4,
    GatewayReauth = 0x03E9,

    // Planet
    PlanetGetChannels = 0x03EB,
    PlanetGetServerList = 0x0FBF,
    PlanetEnterChannel = 0x03ED,
    PlanetLeaveChannel = 0x07E6,

    // List Room
    ListRoomGetCharacter = 0x07D1,
    ListRoomGetRoomList = 0x07D3,
    ListRoomGetPlayerList = 0x07DB,
    ListRoomSyncGems = 0x13A5,
    ListRoomCreateRoom = 0x07D6,
    ListRoomJoinRoom = 0x0BBB,
    ListRoomLeaveRoom = 0x0BBE,
    ListRoomOnChat = 0x07DD,
    ListRoomOnRoomAdded = 0x07D5,
    ListRoomOnRoomRemoved = 0x07D7,
    ListRoomOnRoomStatusChanged = 0x07E4,
    ListRoomOnRoomSkillChanged = 0x0FE9,
    ListRoomOnRoomMusicIdChanged = 0x07E7,
    ListRoomOnRoomNameChanged = 0x077E,
    ListRoomOnRoomMaxPlayerChanged = 0x0207,

    // Room
    RoomOnPlayerEnter = 0x0BC1,
    RoomOnPlayerLeave = 0x0BC2,
    RoomOnChat = 0x0BC4,
    RoomOnReadyChanged = 0x0FA9,
    RoomOnMusicIdChanged = 0x0FA1,
    RoomOnArenaChanged = 0x0FA3,
    RoomOnTeamChanged = 0x0FA5,
    RoomOnSkillChanged = 0x0FB8,
    RoomOnNameChanged = 0x0BB9,

    // Game
    GameStart = 0x0FAB,
    LeaveGame = 0x0FB6,
    GameEvent = 0x0FAF,
    GameFinish = 0x0FB2,
    SubmitScore = 0x0FB1,

    // O2Hook2's Extensions
    RequestVersion = 0xAAAA,
    RejectVersion = 0xAAAC,

    RoomSetModifier = 0xAAB0,
    RoomSetAllModifiers = 0xAAB1,
}
