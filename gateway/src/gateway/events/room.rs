use database::Equipment;

use crate::gateway::{
    commands::EventId,
    room::{ModifierReport, Modifiers, MusicId, MusicIdEntry, RoomArena, RoomDifficulty, RoomSpeed, SkillId, TeamId},
    routes::room::GameStartResult,
};

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnPlayerEnterEventArgs {
    pub slot: u8,
    pub nickname: std::ffi::CString,
    pub level: u32,
    pub gender: u8,
    pub team: TeamId,
    pub unk1: u16,
    pub equipment: Equipment,
    pub music_list: Vec<MusicIdEntry>,
}

#[gateway_derive::event(EventId::RoomOnPlayerEnter)]
async fn on_player_enter(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnPlayerEnterEventArgs>(data) else {
        println!("Failed to downcast event data for RoomPlayerEnter");
        return;
    };
    
    client
        .send_packet(EventId::RoomOnPlayerEnter, data)
        .await
        .expect("Failed to send room player enter response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnPlayerLeaveEventArgs {
    pub slot: u8,
    pub room_master_slot: u8,
    pub premium: u16
}

#[gateway_derive::event(EventId::RoomOnPlayerLeave)]
async fn on_player_leave(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnPlayerLeaveEventArgs>(data) else {
        println!("Failed to downcast event data for RoomPlayerLeave");
        return;
    };

    client
        .send_packet(EventId::RoomOnPlayerLeave, data)
        .await
        .expect("Failed to send room player leave response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnChatEventArgs {
    pub author: std::ffi::CString,
    pub message: std::ffi::CString,
}

#[gateway_derive::event(EventId::RoomOnChat)]
async fn on_room_chat(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnChatEventArgs>(data) else {
        println!("Failed to downcast event data for RoomChat");
        return;
    };

    client
        .send_packet(EventId::RoomOnChat, data)
        .await
        .expect("Failed to send room chat response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnReadyEventArgs {
    pub slot: u8,
    pub ready: bool,
}

#[gateway_derive::event(EventId::RoomOnReadyChanged)]
async fn on_room_player_ready(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnReadyEventArgs>(data) else {
        println!("Failed to downcast event data for RoomPlayerReady");
        return;
    };

    client
        .send_packet(EventId::RoomOnReadyChanged, data)
        .await
        .expect("Failed to send room player ready response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnGameStartEventArgs {
    pub success: GameStartResult,
    pub seed: u32,
}

#[gateway_derive::event(EventId::RoomOnGameStart)]
async fn on_room_game_start(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnGameStartEventArgs>(data) else {
        println!("Failed to downcast event data for RoomGameStart");
        return;
    };

    client
        .send_packet(EventId::RoomOnGameStart, data)
        .await
        .expect("Failed to send room game start response");
}
    
#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnMusicIdChangedEventArgs {
    pub id: MusicId,
    pub difficulty: RoomDifficulty,
    pub speed: RoomSpeed,
}

#[gateway_derive::event(EventId::RoomOnMusicIdChanged)]
async fn on_room_change_music_id(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnMusicIdChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomChangeMusicId");
        return;
    };

    client
        .send_packet(EventId::RoomOnMusicIdChanged, data)
        .await
        .expect("Failed to send set room music response");
}

#[derive(gateway_derive::Event)]
pub struct RoomOnArenaChangedEventArgs {
    pub arena: RoomArena,
    pub random: bool,
}

#[gateway_derive::event(EventId::RoomOnArenaChanged)]
async fn on_room_change_arena(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnArenaChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomChangeArena");
        return;
    };

    client
        .send_packet(EventId::RoomOnArenaChanged, &data.arena)
        .await
        .expect("Failed to send set room arena response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnSkillChangedEventArgs {
    pub ring: Vec<SkillId>,
}

#[gateway_derive::event(EventId::RoomOnSkillChanged)]
async fn on_room_change_ring(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnSkillChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomChangeRing");
        return;
    };

    client
        .send_packet(EventId::RoomOnSkillChanged, data)
        .await
        .expect("Failed to send set room ring response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnTeamChangedEventArgs {
    pub slot: u8,
    pub team: TeamId,
}

#[gateway_derive::event(EventId::RoomOnTeamChanged)]
async fn on_room_change_set_team(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnTeamChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomChangeSetTeam");
        return;
    };

    client
        .send_packet(EventId::RoomOnTeamChanged, data)
        .await
        .expect("Failed to send set room team response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnNameChangedEventArgs {
    pub name: std::ffi::CString,
}

#[gateway_derive::event(EventId::RoomOnNameChanged)]
async fn on_room_name_changed(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(data) = super::downcast::<RoomOnNameChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomNameChanged");
        return;
    };

    client
        .send_packet(EventId::RoomOnNameChanged, data)
        .await
        .expect("Failed to send set room name response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnModifierChangedEventArgs {
    pub modifier: Modifiers,
    pub value: u32,
}

#[gateway_derive::event(EventId::RoomOnModifierChanged)]
async fn on_room_modifier_changed(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(modifier) = super::downcast::<RoomOnModifierChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomModifierChanged");
        return;
    };

    client
        .send_packet(EventId::RoomOnModifierChanged, modifier)
        .await
        .expect("Failed to send set room modifier response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnAllModifiersChangedEventArgs {
    pub modifiers: ModifierReport,
}

#[gateway_derive::event(EventId::RoomOnAllModifiersChanged)]
async fn on_room_all_modifiers_changed(client: &mut super::Client, data: &dyn super::IEventData) {
    let Some(modifiers) = super::downcast::<RoomOnAllModifiersChangedEventArgs>(data) else {
        println!("Failed to downcast event data for RoomAllModifiersChanged");
        return;
    };

    client
        .send_packet(EventId::RoomOnAllModifiersChanged, modifiers)
        .await
        .expect("Failed to send set all room modifiers response");
}
