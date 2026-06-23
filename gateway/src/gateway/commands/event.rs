#[repr(u16)]
#[derive(Debug, Copy, Clone, encoder::StructSerializer, PartialEq, Eq)]
pub enum EventId {
    Disconnect,

    // List Room
    ListRoomOnChat = 0x07DD,                   // Done
    ListRoomOnAddRoom = 0x07D5,                // Done
    ListRoomOnRemoveRoom = 0x07D7,             // Done
    ListRoomOnRoomStatusChanged = 0x07E4,      // Done
    ListRoomOnRoomSkillChanged = 0x07E9,       // Done
    ListRoomOnRoomMusicIdChanged = 0x07E7,     // Done
    ListRoomOnRoomNameChanged = 0x07D8,        // Done
    ListRoomOnRoomPlayerCountChanged = 0x07D9, // Done

    // Room
    RoomOnPlayerEnter = 0x0BBC,    // Done
    RoomOnPlayerLeave = 0x0BBF,    // Done
    RoomOnChat = 0x0BC4,           // Done
    RoomOnReadyChanged = 0x0FA9,   // Done
    RoomOnMusicIdChanged = 0x0FA1, // Done
    RoomOnArenaChanged = 0x0FA3,   // Done
    RoomOnTeamChanged = 0x0FA5,    // Done
    RoomOnSkillChanged = 0x0FB8,   // Done
    RoomOnNameChanged = 0x0BB9,    // Done
    RoomOnGameStart = 0x0FAB,      // Done

    // Game
    GameOnLoadingReady = 0x0FAD,
    GameOnPlayerLeave = 0x0FB6,
    GameOnNoteEvent = 0x0FAF,
    GameOnFinish = 0x0FB2,
    GameOnSubmitScore = 0x0FB1,

    // O2Hook2's Extensions
    RoomOnModifierChanged = 0xAABA,
    RoomOnAllModifiersChanged = 0xAABB,
}
