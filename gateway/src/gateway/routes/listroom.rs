use database::Equipment;

use crate::{
    channel::ChannelCommand,
    gateway::{
        commands::ResponseId,
        events::room::PlayingState,
    },
    itemlist::GameModifierType,
    room::{
        ModifierReport, MusicId, RoomArena, RoomDifficulty, RoomMode, RoomSpeed, RoomStatus,
        RoomWeakHandle, SkillId, TeamId,
    },
    user::{ItemId, User},
};

#[cfg(not(feature = "disable-o2hook2-mod"))]
use crate::gateway::commands::EventId;

#[derive(encoder::StructSerializer)]
pub struct ServerMusicEntry {
    pub songid: u16,
    pub note_count_easy: u16,
    pub note_count_normal: u16,
    pub note_count_hard: u16,
    pub price: u32,
}

// ServerList use u16 as length
pub struct VecU16<T> {
    pub length: u16,
    pub data: Vec<T>,
}

impl<T> VecU16<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            length: data.len() as u16,
            data,
        }
    }
}

impl<T: encoder::StructEncodeImpl> encoder::StructEncodeImpl for VecU16<T> {
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        use byteorder_lite::{LittleEndian, WriteBytesExt as _};

        writer.write_u16::<LittleEndian>(self.length)?;

        for item in &self.data {
            item.impl_encode(writer)?;
        }

        Ok(())
    }
}

#[gateway_derive::route(RequestId::PlanetGetServerList)]
async fn handle_server_list(client: &mut super::Client, _packet: &()) {
    let Some((_, ch)) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot get server list",
            client.id
        );
        return;
    };

    let Ok(list) = ch
        .send::<Vec<ServerMusicEntry>>(ChannelCommand::RequestServerList)
        .await
    else {
        println!(
            "Failed to get server list for channel: {}:{}",
            ch.region(),
            ch.id()
        );
        return;
    };

    let response = VecU16::new(list);

    client
        .send_packet(ResponseId::PlanetGetServerList, &response)
        .await
        .expect("Failed to send server list response");
}

#[gateway_derive::route(RequestId::ListRoomGetClientList)]
async fn handle_client_list(client: &mut super::Client, packet: &Vec<MusicId>) {
    let Some((user_id, ch)) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot get server list",
            client.id
        );
        return;
    };

    ch.send::<()>(ChannelCommand::SetClientList {
        user_id,
        client_ids: packet.clone(),
    })
    .await
    .expect("Failed to send client list to channel");
}

#[gateway_derive::route(RequestId::ListRoomSyncInfo)]
async fn handle_sync_info(client: &mut super::Client, _packet: &()) {
    #[derive(encoder::StructSerializer)]
    struct SyncGoldCurrencyResponse {
        gem: u32,
    }

    let Some((user_id, ch)) = client.channel() else {
        println!("Client {} is not in a channel, cannot sync info", client.id);
        return;
    };

    let Ok(user) = ch
        .send::<User>(ChannelCommand::SyncUserInfo { user_id })
        .await
    else {
        println!(
            "Failed to sync user info for client {}: user not found",
            client.id
        );
        return;
    };

    let response = SyncGoldCurrencyResponse {
        gem: user.info.o2gems,
    };

    client.user = Some(user);

    client
        .send_packet(ResponseId::ListRoomSyncGems, &response)
        .await
        .expect("Failed to send sync currency response");
}

#[derive(Debug, encoder::StructSerializer)]
pub struct UserInfoEntry {
    pub username: std::ffi::CString,
    pub nickname: std::ffi::CString,
    pub level: u32,
}

#[derive(Debug, encoder::StructSerializer, Clone)]
pub struct RoomEntry {
    pub id: u32,
    pub state: RoomStatus,
    pub name: std::ffi::CString,
    pub is_password: bool,
    pub ojn_id: MusicId,
    pub difficulty: RoomDifficulty,
    pub mode: RoomMode,
    pub speed: RoomSpeed,
    pub max_players: u8,
    pub current_players: u8,
    pub min_level: u8,
    pub max_level: u8,
    pub skills: Vec<SkillId>,
    pub premium: u16,
}

const DEFAULT_TITLE: &std::ffi::CStr = c"PlaceHolder Room Name";

impl std::default::Default for RoomEntry {
    fn default() -> Self {
        Self {
            id: 0,
            state: RoomStatus::Waiting,
            name: DEFAULT_TITLE.to_owned(),
            is_password: false,
            ojn_id: MusicId::new(0),
            difficulty: RoomDifficulty::Easy,
            mode: RoomMode::Solo,
            speed: RoomSpeed::Speed05,
            max_players: 0,
            current_players: 0,
            min_level: 0,
            max_level: 0,
            skills: Vec::new(),
            premium: 0,
        }
    }
}

#[gateway_derive::route(RequestId::ListRoomGetRoomList)]
async fn handle_get_room_list(client: &mut super::Client, _packet: &()) {
    // For some reason, the game didnt send the GetPlayerList request,
    // but it still expects a response for it, so we just send the list
    // in the GetRoom response, which seems to work fine.
    // Maybe the client is just weirdly coded and expects the player list in the GetRoom response?

    let Some((_, channel)) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot get room list",
            client.id
        );
        return;
    };

    let get_rooms_fut = channel.send::<Vec<RoomEntry>>(ChannelCommand::GetRooms);
    let get_users_fut = channel.send::<Vec<UserInfoEntry>>(ChannelCommand::GetUsers);

    let (Ok(rooms), Ok(users)) = tokio::join!(get_rooms_fut, get_users_fut) else {
        println!(
            "Failed to get room or user list for channel: {}:{}",
            channel.region(),
            channel.id()
        );
        return;
    };

    client
        .send_packet(ResponseId::ListRoomGetPlayerList, &users)
        .await
        .expect("Failed to send player list response");

    client
        .send_packet(ResponseId::ListRoomGetRoomList, &rooms)
        .await
        .expect("Failed to send room list response");
}

#[derive(Debug, Clone, encoder::StructSerializer)]
pub struct EffectEntry {
    pub id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, encoder::StructSerializer)]
pub struct CharacterResponse {
    pub invalid: u32,
    pub nickname: std::ffi::CString,
    pub gender: u8,
    pub gem: u32,
    pub mcash: u32,
    pub o2cash: u32,
    pub level: u32,
    pub win: u32,
    pub lose: u32,
    pub draw: u32,
    pub play_count: u32,
    pub experience: u32,
    pub is_admin: bool,
    pub equipment: Equipment,
    pub inventory: [ItemId; 30],
    pub padding: [u32; 5],
    pub effects: Vec<EffectEntry>,
}

#[gateway_derive::route(RequestId::ListRoomGetCharacter)]
async fn handle_get_character(client: &mut super::Client, _packet: &()) {
    let Some(user) = client.user() else {
        println!(
            "Client {} is not logged in, cannot get character info",
            client.id
        );
        return;
    };

    let response = {
        let lists = crate::itemlist::get();

        let mut effects = Vec::new();
        for i in user.inventory.iter().filter(|item| item.id != 0) {
            let Some(item) = lists.get_item(i.id) else {
                continue;
            };

            if item.modifier_type != GameModifierType::None {
                effects.push(EffectEntry {
                    id: i.id,
                    amount: i.amount,
                });
            }
        }

        CharacterResponse {
            invalid: 0,
            nickname: std::ffi::CString::new(user.nickname()).unwrap_or_default(),
            gender: 1,
            gem: user.info.o2gems,
            mcash: user.info.mcash,
            o2cash: user.info.point,
            level: user.level(),
            win: 0,
            lose: 0,
            draw: 0,
            play_count: 0,
            experience: 0,
            is_admin: true,
            equipment: user.equipment(),
            inventory: user.inventory(),
            padding: [0; 5],
            effects,
        }
    };

    client
        .send_packet(ResponseId::ListRoomGetCharacter, &response)
        .await
        .expect("Failed to send character response");
}

#[derive(Debug, encoder::StructDeserializer)]
pub enum JoinState {
    Ready = 0,
}

#[derive(encoder::StructDeserializer)]
pub struct JoinRoomRequest {
    pub room_id: u32,
    pub state: JoinState,
    pub password: std::ffi::CString,
}

#[repr(i32)]
#[derive(Copy, Clone, encoder::StructSerializer, PartialEq, Eq)]
pub enum JoinErrorCode {
    Success = 0,
    GenericError = -1,
    InvalidMode = -2,
    InvalidPassword = -3,
    InProgress = -4,
    RoomFull = -5,
    RoomFull2 = -6,
    NoPass = -7,
}

#[derive(encoder::StructSerializer)]
pub struct MemberInfo {
    pub nickname: std::ffi::CString,
    pub level: u32,
    pub gender: u8,
    pub is_room_master: bool,
    pub color: TeamId,
    pub ready: bool,
    pub state: PlayingState,
    pub equipment: Equipment,
    pub list: Vec<MusicId>,
}

#[derive(encoder::StructSerializer, Clone, Copy, PartialEq, Eq)]
pub enum PositionStatus {
    Empty = 0,
    Occupied = 1,
    Locked = 2,
}

pub struct PositionInfo {
    pub position: u8,
    pub status: PositionStatus,
    pub member: Option<MemberInfo>,
}

impl encoder::StructEncodeImpl for PositionInfo {
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        use byteorder_lite::WriteBytesExt as _;

        writer.write_u8(self.position)?;
        writer.write_u32::<byteorder_lite::LittleEndian>(self.status as u32)?;

        if self.status == PositionStatus::Occupied {
            if let Some(member) = &self.member {
                member.impl_encode(writer)?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Position is occupied but member info is missing",
                ));
            }
        }

        Ok(())
    }
}

#[derive(encoder::StructSerializer)]
pub struct JoinRoomResponse {
    pub result: JoinErrorCode,
    pub slot: u8,
    pub team: TeamId,
    pub name: std::ffi::CString,
    pub music_id: MusicId,
    pub arena: RoomArena,
    pub mode: RoomMode,
    pub diffculty: RoomDifficulty,
    pub speed: RoomSpeed,
    pub user_count: u32,
    pub slots: [PositionInfo; 7],
    pub skills: Vec<SkillId>,
    pub premium: u16,
}

const DEFAULT_ROOM_NAME: &std::ffi::CStr = c"PlaceHolder Room Name";

impl JoinRoomResponse {
    pub fn new() -> Self {
        Self {
            result: JoinErrorCode::GenericError,
            slot: 0,
            team: TeamId::Blue,
            name: DEFAULT_ROOM_NAME.to_owned(),
            music_id: MusicId::new(0),
            arena: RoomArena::ARENA1,
            mode: RoomMode::Solo,
            diffculty: RoomDifficulty::Easy,
            speed: RoomSpeed::Speed05,
            user_count: 0,
            slots: [const {
                PositionInfo {
                    position: 0,
                    status: PositionStatus::Empty,
                    member: None,
                }
            }; 7],
            skills: Vec::new(),
            premium: 0,
        }
    }
}

impl JoinRoomResponse {
    pub fn user_not_found() -> Self {
        Self {
            result: JoinErrorCode::GenericError,
            ..Self::new()
        }
    }

    pub fn room_not_found() -> Self {
        Self {
            result: JoinErrorCode::GenericError,
            ..Self::new()
        }
    }

    pub fn invalid_password() -> Self {
        Self {
            result: JoinErrorCode::InvalidPassword,
            ..Self::new()
        }
    }

    pub fn room_full() -> Self {
        Self {
            result: JoinErrorCode::RoomFull,
            ..Self::new()
        }
    }
}

#[gateway_derive::route(RequestId::ListRoomJoinRoom)]
async fn handle_join_room(client: &mut super::Client, packet: &JoinRoomRequest) {
    let Some((user_id, ch)) = client.channel() else {
        println!("Client {} is not in a channel, cannot join room", client.id);
        return;
    };

    let password_option = if packet.password.to_bytes_with_nul().len() > 0 {
        Some(packet.password.to_string_lossy().to_string())
    } else {
        None
    };

    let Ok((response, data)) = ch
        .send::<(JoinRoomResponse, Option<(RoomWeakHandle, ModifierReport)>)>(
            ChannelCommand::JoinRoom {
                user_id,
                room_id: packet.room_id,
                password: password_option,
            },
        )
        .await
    else {
        println!(
            "Failed to join room {} for client {}",
            packet.room_id, client.id
        );
        return;
    };

    client
        .send_packet(ResponseId::ListRoomJoinRoom, &response)
        .await
        .expect("Failed to send join room response");

    if let Some((handle, modifier)) = data {
        client.room_handle = Some(handle);

        #[cfg(not(feature = "disable-o2hook2-mod"))]
        client
            .send_packet(EventId::RoomOnAllModifiersChanged, &modifier)
            .await
            .expect("Failed to send modifier report response");

        #[cfg(feature = "disable-o2hook2-mod")]
        let _ = modifier; // To prevent unused variable warning when the feature is disabled
    }
}

#[derive(Clone, Copy, encoder::StructSerializer)]
pub enum LeaveErrorCode {
    Success = 0,
}

#[gateway_derive::route(RequestId::ListRoomLeaveRoom)]
async fn handle_leave_room(client: &mut super::Client, _packet: &()) {
    let Some((user_id, ch)) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot leave room",
            client.id
        );
        return;
    };

    if ch
        .send::<()>(ChannelCommand::LeaveRoom { user_id })
        .await
        .is_err()
    {
        println!("Failed to send leave room command for client {}", client.id);
        return;
    }

    client.room_handle = None;

    client
        .send_packet(ResponseId::ListRoomLeaveRoom, &LeaveErrorCode::Success)
        .await
        .expect("Failed to send leave room response");
}
