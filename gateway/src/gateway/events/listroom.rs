use crate::{
    room::{MusicId, RoomDifficulty, RoomMode, RoomSpeed, RoomStatus, SkillId},
    gateway::commands::EventId,
};

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomChatEventArgs {
    pub author: std::ffi::CString,
    pub message: std::ffi::CString,
}

#[gateway_derive::event(EventId::ListRoomOnChat)]
async fn on_chat(client: &mut super::Client, data: &ListRoomAddRoomEventArgs) {
    client
        .send_packet(EventId::ListRoomOnChat, data)
        .await
        .expect("Failed to send room chat response");
}

#[derive(Debug, gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomAddRoomEventArgs {
    pub id: u32,
    pub title: std::ffi::CString,
    pub mode: RoomMode,
    pub has_password: bool,
    pub min_level: u8,
    pub max_level: u8,
    pub premium: u16,
}

#[gateway_derive::event(EventId::ListRoomOnAddRoom)]
async fn on_room_add(client: &mut super::Client, data: &ListRoomAddRoomEventArgs) {
    client
        .send_packet(EventId::ListRoomOnAddRoom, data)
        .await
        .expect("Failed to send add waiting room response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomRemoveRoomEventArgs {
    pub id: u32,
}

#[gateway_derive::event(EventId::ListRoomOnRemoveRoom)]
async fn on_remove_room(client: &mut super::Client, data: &ListRoomRemoveRoomEventArgs) {
    client
        .send_packet(EventId::ListRoomOnRemoveRoom, data)
        .await
        .expect("Failed to send remove waiting room response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomChangeRoomStatusEventArgs {
    pub id: u32,
    pub status: RoomStatus,
}

#[gateway_derive::event(EventId::ListRoomOnRoomStatusChanged)]
async fn on_room_status_changed(
    client: &mut super::Client,
    data: &ListRoomChangeRoomStatusEventArgs,
) {
    client
        .send_packet(EventId::ListRoomOnRoomStatusChanged, data)
        .await
        .expect("Failed to send waiting room status changed response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomChangeRoomMusicIdEventArgs {
    pub id: u32,
    pub music_id: MusicId,
    pub difficulty: RoomDifficulty,
    pub speed: RoomSpeed,
}

#[gateway_derive::event(EventId::ListRoomOnRoomMusicIdChanged)]
async fn on_room_music_id_changed(
    client: &mut super::Client,
    data: &ListRoomChangeRoomMusicIdEventArgs,
) {
    client
        .send_packet(EventId::ListRoomOnRoomMusicIdChanged, data)
        .await
        .expect("Failed to send waiting room music id changed response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomChangeRoomNameEventArgs {
    pub id: u32,
    pub name: std::ffi::CString,
}

#[gateway_derive::event(EventId::ListRoomOnRoomNameChanged)]
async fn on_room_name_changed(client: &mut super::Client, data: &ListRoomChangeRoomNameEventArgs) {
    client
        .send_packet(EventId::ListRoomOnRoomNameChanged, data)
        .await
        .expect("Failed to send waiting room name changed response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomChangeRoomMaxPlayerEventArgs {
    pub id: u32,
    pub max_player: u8,
    pub current_player: u8,
    pub premium: u16,
}

#[gateway_derive::event(EventId::ListRoomOnRoomPlayerCountChanged)]
async fn on_room_max_player_changed(
    client: &mut super::Client,
    data: &ListRoomChangeRoomMaxPlayerEventArgs,
) {
    client
        .send_packet(EventId::ListRoomOnRoomPlayerCountChanged, data)
        .await
        .expect("Failed to send waiting room max player changed response");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct ListRoomChangeRoomSkillEventArgs {
    pub id: u32,
    pub skill: Vec<SkillId>,
}

#[gateway_derive::event(EventId::ListRoomOnRoomSkillChanged)]
async fn on_room_skill_changed(
    client: &mut super::Client,
    data: &ListRoomChangeRoomSkillEventArgs,
) {
    client
        .send_packet(EventId::ListRoomOnRoomSkillChanged, data)
        .await
        .expect("Failed to send waiting room skill changed response");
}
