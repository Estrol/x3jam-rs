use crate::{
    channel::ChannelCommand, gateway::commands::ResponseId, room::{
        ModifierReport, Modifiers, MusicId, RoomArena, RoomCommand, RoomDifficulty, RoomMode, RoomSpeed, RoomWeakHandle, SkillId, TeamId,
    },
};

#[derive(Debug, Copy, Clone, encoder::StructSerializer)]
pub enum CreateRoomResult {
    Success = 0,
    Full = 1,
}

#[derive(encoder::StructDeserializer)]
struct CreateRoomRequest {
    title: std::ffi::CString,
    mode: RoomMode,
    password: Option<std::ffi::CString>,
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
        println!("Client is not in a channel");
        return;
    };

    let title = request.title.to_string_lossy().to_string();
    let password = request
        .password
        .clone()
        .map(|p| p.to_string_lossy().to_string());
    let mode = request.mode;
    let min_level = request.min_level;
    let max_level = request.max_level;

    let Ok((result, data)) = channel
        .send::<(CreateRoomResult, Option<RoomWeakHandle>)>(ChannelCommand::CreateRoom {
            user_id,
            name: title,
            password,
            mode,
            min_level,
            max_level,
        })
        .await
    else {
        println!("Failed to send create room command to channel");
        return;
    };

    let response = CreateRoomResponse {
        result,
        room_id: data.as_ref().map(|handle| handle.id).unwrap_or(0),
        premium: false,
    };

    client.room_handle = data;

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
    let Some((_, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetMusicId {
            music_id: packet.id,
            difficulty: packet.difficulty,
            speed: packet.speed,
        })
        .await
        .expect("Failed to send set music id command to channel");
}

#[gateway_derive::route(RequestId::RoomSetArena)]
async fn handle_set_room_arena(client: &mut super::Client, packet: &RoomArena) {
    let Some((_, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetArena {
            arena: *packet,
        })
        .await
        .expect("Failed to send set arena command to channel");
}

#[gateway_derive::route(RequestId::RoomSetTeam)]
async fn handle_set_room_team(client: &mut super::Client, packet: &TeamId) {
    let Some((user_id, channel)) = client.room() else {
        println!("Client is not in a channel");
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
    let Some((_, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetSkills {
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
        println!("Client is not in a channel");
        return;
    };

    let result = channel
        .send::<GameStartResult>(RoomCommand::StartGame {
            user_id,
        })
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
async fn handle_change_title(client: &mut super::Client, packet: &std::ffi::CString) {
    let Some((_, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    let title = packet.to_string_lossy().to_string();

    channel
        .send::<()>(RoomCommand::SetName {
            name: title,
        })
        .await
        .expect("Failed to send set name command to channel");
}

#[gateway_derive::route(RequestId::RoomSetReady)]
async fn handle_set_ready(client: &mut super::Client, _packet: &()) {
    let Some((user_id, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetReady { user_id })
        .await
        .expect("Failed to send set ready command to channel");
}

#[gateway_derive::route(RequestId::RoomChat)]
async fn handle_room_chat(client: &mut super::Client, packet: &std::ffi::CString) {
    let Some((user_id, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    let message = packet.to_string_lossy().to_string();

    channel
        .send::<()>(RoomCommand::RoomChat { user_id, message })
        .await
        .expect("Failed to send room chat command to channel");
}

#[derive(encoder::StructDeserializer)]
struct SetModifierRequest {
    modifier: Modifiers,
    value: u32,
}

#[gateway_derive::route(RequestId::RoomSetModifier)]
async fn handle_set_modifier(client: &mut super::Client, packet: &SetModifierRequest) {
    let Some((_, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetModifier {
            modifier: packet.modifier,
            value: packet.value,
        })
        .await
        .expect("Failed to send set modifier command to channel");
}

#[gateway_derive::route(RequestId::RoomSetAllModifiers)]
async fn handle_set_all_modifiers(client: &mut super::Client, packet: &ModifierReport) {
    let Some((_, channel)) = client.room() else {
        println!("Client is not in a channel");
        return;
    };

    channel
        .send::<()>(RoomCommand::SetAllModifiers {
            modifiers: packet.clone(),
        })
        .await
        .expect("Failed to send set all modifiers command to channel");
}
