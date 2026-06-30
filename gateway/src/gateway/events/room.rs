use database::Equipment;

use crate::{
    room::{
        ModifierReport, Modifiers, MusicId, MusicIdEntry, RoomArena, RoomDifficulty, RoomSpeed,
        SkillId, TeamId,
    },
    gateway::{commands::EventId, routes::room::GameStartResult},
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
async fn on_player_enter(client: &mut super::Client, data: &RoomOnPlayerEnterEventArgs) {
    client
        .send_packet(EventId::RoomOnPlayerEnter, data)
        .await
        .expect("Failed to send room player enter response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnPlayerLeaveEventArgs {
    pub slot: u8,
    pub room_master_slot: u8,
    pub premium: u16,
}

#[gateway_derive::event(EventId::RoomOnPlayerLeave)]
async fn on_player_leave(client: &mut super::Client, data: &RoomOnPlayerLeaveEventArgs) {
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
async fn on_room_chat(client: &mut super::Client, data: &RoomOnChatEventArgs) {
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
async fn on_room_player_ready(client: &mut super::Client, data: &RoomOnReadyEventArgs) {
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
async fn on_room_game_start(client: &mut super::Client, data: &RoomOnGameStartEventArgs) {
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
async fn on_room_change_music_id(client: &mut super::Client, data: &RoomOnMusicIdChangedEventArgs) {
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
async fn on_room_change_arena(client: &mut super::Client, data: &RoomOnArenaChangedEventArgs) {
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
async fn on_room_change_ring(client: &mut super::Client, data: &RoomOnSkillChangedEventArgs) {
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
async fn on_room_change_set_team(client: &mut super::Client, data: &RoomOnTeamChangedEventArgs) {
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
async fn on_room_name_changed(client: &mut super::Client, data: &RoomOnNameChangedEventArgs) {
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
async fn on_room_modifier_changed(
    client: &mut super::Client,
    data: &RoomOnModifierChangedEventArgs,
) {
    client
        .send_packet(EventId::RoomOnModifierChanged, data)
        .await
        .expect("Failed to send set room modifier response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct RoomOnAllModifiersChangedEventArgs {
    pub modifiers: ModifierReport,
}

#[gateway_derive::event(EventId::RoomOnAllModifiersChanged)]
async fn on_room_all_modifiers_changed(
    client: &mut super::Client,
    data: &RoomOnAllModifiersChangedEventArgs,
) {
    client
        .send_packet(EventId::RoomOnAllModifiersChanged, data)
        .await
        .expect("Failed to send set all room modifiers response");
}
