use std::sync::Arc;

use crate::gateway::{
    channel::CreateError, commands::ResponseId, room::{Modifiers, MusicId, RoomArena, RoomDifficulty, RoomMode, RoomSpeed, SkillId, TeamId, modifier::ModifierReport}
};

#[derive(Debug, Copy, Clone, encoder::StructSerializer)]
pub enum CreateRoomResult {
    Success = 0,
    Full = 1,
}

#[gateway_derive::route(RequestId::ListRoomCreateRoom)]
async fn handle_create_room(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(user) = client.user.as_ref() else {
        println!("Client is not logged in");
        return;
    };

    let Some(channel) = client.channel() else {
        println!("Client is not in a channel");
        return;
    };

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

    match super::parse_request::<CreateRoomRequest>(&packet.body) {
        Ok(request) => {
            let title = request.title.to_string_lossy().to_string();
            let password = request.password.map(|p| p.to_string_lossy().to_string());
            let mode = request.mode;
            let min_level = request.min_level;
            let max_level = request.max_level;

            let room = {
                let mut channel = channel.lock().await;

                channel
                    .create_room(user, title.clone(), password.clone(), mode, min_level, max_level)
                    .await
            };
            
            match room
            {
                Ok((id, room)) => {
                    let response = CreateRoomResponse {
                        result: CreateRoomResult::Success,
                        room_id: id,
                        premium: false,
                    };

                    client.room = Some(Arc::downgrade(&room));

                    client
                        .send_packet(ResponseId::ListRoomCreateRoom, &response)
                        .await
                        .expect("Failed to send create room response");
                }
                Err(CreateError::ChannelFull) => {
                    let response = CreateRoomResponse {
                        result: CreateRoomResult::Full,
                        room_id: 0,
                        premium: false,
                    };

                    client
                        .send_packet(ResponseId::ListRoomCreateRoom, &response)
                        .await
                        .expect("Failed to send create room response");
                }
            }
        }
        Err(e) => {
            println!("[Error] Failed to parse create room request: {}", e);
        }
    }
}

#[derive(encoder::StructDeserializer, encoder::StructSerializer)]
pub struct SetRoomMusic {
    pub id: MusicId,
    pub difficulty: RoomDifficulty,
    pub speed: RoomSpeed,
}

#[gateway_derive::route(RequestId::RoomSetMusicId)]
async fn handle_set_room_music(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    match super::parse_request::<SetRoomMusic>(&packet.body) {
        Ok(request) => {
            let mut room = room.lock().await;
            room.set_song_id(request.id, request.difficulty, request.speed)
                .await;
        }
        Err(e) => {
            println!("[Error] Failed to parse set room music request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::RoomSetArena)]
async fn handle_set_room_arena(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    match super::parse_request::<RoomArena>(&packet.body) {
        Ok(arena) => {
            let mut room = room.lock().await;
            room.set_arena(arena, arena.contains(RoomArena::RANDOM_FLAG));
        }
        Err(e) => {
            println!("[Error] Failed to parse set room arena request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::RoomSetTeam)]
async fn handle_set_room_team(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    let Some(user) = client.user.as_ref() else {
        println!("Client is not logged in");
        return;
    };

    match super::parse_request::<TeamId>(&packet.body) {
        Ok(team_id) => {
            let mut room = room.lock().await;
            room.set_team(user, team_id).await;
        }
        Err(e) => {
            println!("[Error] Failed to parse set room team request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::RoomSetSkill)]
async fn handle_set_room_ring(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    match super::parse_request::<Vec<SkillId>>(&packet.body) {
        Ok(ring) => {
            let mut room = room.lock().await;
            room.set_ring(ring).await;
        }
        Err(e) => {
            println!("[Error] Failed to parse set room ring request: {}", e);
        }
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, encoder::StructSerializer)]
pub enum GameStartResult {
    Success = 0,
    NotAllReady = 1,
    NotHost = 2,
}

#[gateway_derive::route(RequestId::GameStart)]
async fn handle_start_game(client: &mut super::Client, _packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    let Some(user) = client.user.as_ref() else {
        println!("Client is not logged in");
        return;
    };

    let result = {
        let mut room = room.lock().await;

        if !room.is_host(user) {
            GameStartResult::NotHost
        } else if !room.is_all_ready() {
            GameStartResult::NotAllReady
        } else {
            return room.game_start().await;
        }
    };

    client
        .send_packet(ResponseId::GameStart, &result)
        .await
        .expect("Failed to send game start response");
}

#[gateway_derive::route(RequestId::RoomNameChange)]
async fn handle_change_title(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    match super::parse_request::<std::ffi::CString>(&packet.body) {
        Ok(title) => {
            let title = title.to_string_lossy().to_string();

            let mut room = room.lock().await;
            room.set_name(&title).await;
        }
        Err(e) => {
            println!("[Error] Failed to parse change title request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::RoomSetReady)]
async fn handle_set_ready(client: &mut super::Client, _packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    let Some(user) = client.user.as_ref() else {
        println!("Client is not logged in");
        return;
    };

    let mut room = room.lock().await;
    room.set_ready(user).await;
}

#[gateway_derive::route(RequestId::RoomChat)]
async fn handle_room_chat(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    let Some(user) = client.user.as_ref() else {
        println!("Client is not logged in");
        return;
    };

    match super::parse_request::<std::ffi::CString>(&packet.body) {
        Ok(message) => {
            let message = message.to_string_lossy().to_string();

            let mut room = room.lock().await;
            room.on_chat(user, &message).await;
        }
        Err(e) => {
            println!("[Error] Failed to parse room chat request: {}", e);
        }
    }
}

#[derive(encoder::StructDeserializer)]
struct SetModifierRequest {
    modifier: Modifiers,
    value: u32,
}

#[gateway_derive::route(RequestId::RoomSetModifier)]
async fn handle_set_modifier(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    match super::parse_request::<SetModifierRequest>(&packet.body) {
        Ok(request) => {
            let mut room = room.lock().await;
            room.set_modifier(request.modifier, request.value).await;
        }
        Err(e) => {
            println!("[Error] Failed to parse set modifier request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::RoomSetAllModifiers)]
async fn handle_set_all_modifiers(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    match super::parse_request::<ModifierReport>(&packet.body) {
        Ok(modifiers) => {
            let mut room = room.lock().await;
            room.set_all_modifiers(modifiers).await;
        }
        Err(e) => {
            println!("[Error] Failed to parse set all modifiers request: {}", e);
        }
    }
}
