use encoder::stringutil::CStrEx;

use crate::{
    channel::ChannelCommand,
    gateway::{commands::ResponseId, events::room::PlayingState},
    room::{
        ModifierReport, Modifiers, MusicId, RoomArena, RoomCommand, RoomDifficulty, RoomMode,
        RoomSpeed, RoomWeakHandle, SkillId, TeamId,
    },
};

#[cfg(not(feature = "disable-o2hook2-mod"))]
use crate::gateway::commands::EventId;

#[derive(Debug, Copy, Clone, encoder::StructSerializer)]
pub enum CreateRoomResult {
    Success = 0,
    Full = 1,
}

#[derive(encoder::StructDeserializer)]
struct CreateRoomRequest {
    title: CStrEx,
    mode: RoomMode,
    password: Option<CStrEx>,
    min_level: u8,
    max_level: u8,
}

#[derive(encoder::StructSerializer)]
struct CreateRoomResponse {
    result: CreateRoomResult,
    room_id: u32,
    premium: bool,
}

#[gateway_derive::route(RequestId::ListRoomCreateRoom)]
async fn handle_create_room(client: &mut super::Client, request: &CreateRoomRequest) {
    let Some((user_id, channel)) = client.channel() else {
        log::info!("Client is not in a channel");
        return;
    };

    let title = request.title.to_string();
    let password = request
        .password
        .clone()
        .map(|p| p.to_string());

    let mode = request.mode;
    let min_level = request.min_level;
    let max_level = request.max_level;

    let Ok((result, data)) = channel
        .send::<(CreateRoomResult, Option<(RoomWeakHandle, ModifierReport)>)>(
            ChannelCommand::CreateRoom {
                user_id,
                name: title,
                password,
                mode,
                min_level,
                max_level,
            },
        )
        .await
    else {
        log::info!("Failed to send create room command to channel");
        return;
    };

    let Some((room_handle, modifier_report)) = data else {
        client
            .send_packet(ResponseId::ListRoomCreateRoom, &result)
            .await
            .expect("Failed to send create room response");

        return;
    };

    let id = room_handle.id;

    client.room_handle = Some(room_handle);

    #[cfg(not(feature = "disable-o2hook2-mod"))]
    client
        .send_packet(EventId::RoomOnAllModifiersChanged, &modifier_report)
        .await
        .expect("Failed to send set all room modifiers response");

    #[cfg(feature = "disable-o2hook2-mod")]
    let _ = modifier_report; // To prevent unused variable warning when the feature is disabled

    let response = CreateRoomResponse {
        result,
        room_id: id,
        premium: false,
    };

    client
        .send_packet(ResponseId::ListRoomCreateRoom, &response)
        .await
        .expect("Failed to send create room response");
}

#[derive(encoder::StructDeserializer, encoder::StructSerializer)]
pub struct SetRoomMusic {
    pub id: MusicId,
    pub difficulty: RoomDifficulty,
    pub speed: RoomSpeed,
}

#[gateway_derive::route(RequestId::RoomSetMusicId)]
async fn handle_set_room_music(client: &mut super::Client, packet: &SetRoomMusic) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetMusicId {
            user_id,
            music_id: packet.id,
            difficulty: packet.difficulty,
            speed: packet.speed,
        })
        .await
        .expect("Failed to send set music id command to channel");
}

#[gateway_derive::route(RequestId::RoomSetArena)]
async fn handle_set_room_arena(client: &mut super::Client, packet: &RoomArena) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetArena {
            user_id,
            arena: *packet,
        })
        .await
        .expect("Failed to send set arena command to channel");
}

#[gateway_derive::route(RequestId::RoomSetTeam)]
async fn handle_set_room_team(client: &mut super::Client, packet: &TeamId) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetTeam {
            user_id,
            team: *packet,
        })
        .await
        .expect("Failed to send set team command to channel");
}

#[gateway_derive::route(RequestId::RoomSetSkill)]
async fn handle_set_room_ring(client: &mut super::Client, packet: &Vec<SkillId>) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetSkills {
            user_id,
            skills: packet.clone(),
        })
        .await
        .expect("Failed to send set skills command to channel");
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, encoder::StructSerializer, PartialEq, Eq)]
pub enum GameStartResult {
    Success = 0,
    NotAllReady = 1,
    NotHost = 2,
}

#[gateway_derive::route(RequestId::GameStart)]
async fn handle_start_game(client: &mut super::Client, _packet: &()) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    let result = channel
        .send::<GameStartResult>(RoomCommand::StartGame { user_id })
        .await
        .expect("Failed to send start game command to channel");

    if result != GameStartResult::Success {
        client
            .send_packet(ResponseId::GameStart, &result)
            .await
            .expect("Failed to send game start response");
    }
}

#[gateway_derive::route(RequestId::RoomNameChange)]
async fn handle_change_title(client: &mut super::Client, packet: &CStrEx) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    let title = packet.to_string();

    channel
        .send::<()>(RoomCommand::SetName {
            user_id,
            name: title,
        })
        .await
        .expect("Failed to send set name command to channel");
}

#[gateway_derive::route(RequestId::RoomSetReady)]
async fn handle_set_ready(client: &mut super::Client, _packet: &()) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetReady { user_id })
        .await
        .expect("Failed to send set ready command to channel");
}

#[gateway_derive::route(RequestId::RoomChat)]
async fn handle_room_chat(client: &mut super::Client, packet: &CStrEx) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    let message = packet.to_string();

    channel
        .send::<()>(RoomCommand::RoomChat { user_id, message })
        .await
        .expect("Failed to send room chat command to channel");
}

#[derive(encoder::StructDeserializer)]
#[allow(dead_code)]
struct SetModifierRequest {
    modifier: Modifiers,
    value: u32,
}

#[cfg(not(feature = "disable-o2hook2-mod"))]
#[gateway_derive::route(RequestId::RoomSetModifier)]
async fn handle_set_modifier(client: &mut super::Client, packet: &SetModifierRequest) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetModifier {
            user_id,
            modifier: packet.modifier,
            value: packet.value,
        })
        .await
        .expect("Failed to send set modifier command to channel");
}

#[cfg(not(feature = "disable-o2hook2-mod"))]
#[gateway_derive::route(RequestId::RoomSetAllModifiers)]
async fn handle_set_all_modifiers(client: &mut super::Client, packet: &ModifierReport) {
    let Some((user_id, channel)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetAllModifiers {
            user_id,
            modifiers: packet.clone(),
        })
        .await
        .expect("Failed to send set all modifiers command to channel");
}

#[gateway_derive::route(RequestId::RoomSlotToggle)]
async fn handle_toggle_slot(client: &mut super::Client, packet: &u8) {
    let Some((user_id, room)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    let _ = room
        .send::<()>(RoomCommand::ToggleSlot {
            user_id,
            slot: *packet as usize,
        })
        .await;
}

#[gateway_derive::route(RequestId::RoomSetMusicState)]
async fn handle_set_music_state(client: &mut super::Client, packet: &PlayingState) {
    let Some((user_id, room)) = client.room() else {
        log::info!("Client is not in a channel");
        return;
    };

    let _ = room
        .send::<()>(RoomCommand::SetMusicState {
            user_id,
            state: *packet,
        })
        .await;
}
