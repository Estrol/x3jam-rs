use std::sync::Arc;

use database::Equipment;

use crate::gateway::{
    channel::JoinError,
    commands::{EventId, ResponseId},
    itemlist::GameModifierType,
    room::{MusicId, RoomArena, RoomDifficulty, RoomMode, RoomSpeed, RoomStatus, SkillId, TeamId, music::MusicIdEntry},
    user::ItemId,
};

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
async fn handle_server_list(client: &mut super::Client, _packet: &mut super::Packet) {
    let Some(channel) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot get server list",
            client.id
        );
        return;
    };

    let list = {
        let channel = channel.lock().await;

        channel.get_music_list()
    };

    let list = VecU16::new(list);

    client
        .send_packet(ResponseId::PlanetGetServerList, &list)
        .await
        .expect("Failed to send server list response");
}

#[gateway_derive::route(RequestId::ListRoomGetPlayerList)]
async fn handle_client_list(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(channel) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot get client list",
            client.id
        );
        return;
    };

    match super::parse_request::<Vec<MusicIdEntry>>(&packet.body) {
        Ok(client_ids) => {
            let compatible_client_ids = {
                let server_lists = channel.lock().await.list.clone();

                let mut compatible_client_ids = Vec::new();

                for music_id in client_ids.iter() {
                    let id = music_id.songid.0 as i32;
                    if server_lists.iter().any(|entry| entry.songid == id as i32) {
                        compatible_client_ids.push(*music_id);
                    }
                }

                compatible_client_ids
            };

            println!(
                "Client {} requested client list with {} entries, {} of them are compatible",
                client.id,
                client_ids.len(),
                compatible_client_ids.len()
            );

            let Some(user) = client.user() else {
                println!(
                    "Client {} is not logged in, cannot get client list",
                    client.id
                );
                return;
            };

            user.set_music_list(compatible_client_ids);
        }
        Err(e) => {
            println!("[Error] Failed to parse client list request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::ListRoomSyncInfo)]
async fn handle_sync_info(client: &mut super::Client, _packet: &mut super::Packet) {
    #[derive(encoder::StructSerializer)]
    struct SyncGoldCurrencyResponse {
        gem: u32,
    }

    if let Some(user) = client.user() {
        // Since we use cached value, we use this route to sync the user info with the database.
        user.sync().await;

        let response = SyncGoldCurrencyResponse {
            gem: user.info.o2gems,
        };

        client
            .send_packet(ResponseId::ListRoomSyncGems, &response)
            .await
            .expect("Failed to send sync currency response");
    }
}

#[gateway_derive::route(RequestId::ShopLeave)]
async fn enter_shop(_client: &mut super::Client, _packet: &mut super::Packet) {
    // The game sends this packet when the player opens the shop, but it doesn't seem to expect a response for it, so we just ignore it.
}

#[gateway_derive::route(RequestId::ShopEnter)]
async fn leave_shop(_client: &mut super::Client, _packet: &mut super::Packet) {
    // The game sends this packet when the player closes the shop, but it doesn't seem to expect a response for it, so we just ignore it.
}

#[derive(encoder::StructSerializer)]
pub struct UserInfoEntry {
    pub username: std::ffi::CString,
    pub nickname: std::ffi::CString,
    pub level: u32,
}

#[derive(encoder::StructSerializer, Clone)]
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
            ojn_id: MusicId(0),
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
async fn handle_get_room_list(client: &mut super::Client, _packet: &mut super::Packet) {
    // For some reason, the game didnt send the GetPlayerList request,
    // but it still expects a response for it, so we just send the list
    // in the GetRoom response, which seems to work fine.
    // Maybe the client is just weirdly coded and expects the player list in the GetRoom response?

    let Some(channel) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot get room list",
            client.id
        );
        return;
    };

    let (users, rooms) = {
        let channel = channel.lock().await;

        tokio::join!(channel.get_users(), channel.get_rooms())
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

#[gateway_derive::route(RequestId::ListRoomGetCharacter)]
async fn handle_get_character(client: &mut super::Client, _packet: &mut super::Packet) {
    let Some(user) = client.user() else {
        println!(
            "Client {} is not logged in, cannot get character info",
            client.id
        );
        return;
    };

    #[derive(Debug, encoder::StructSerializer)]
    pub struct EffectEntry {
        id: u32,
        amount: u32,
    }

    #[derive(Debug, encoder::StructSerializer)]
    pub struct CharacterResponse {
        invalid: u32,
        nickname: std::ffi::CString,
        gender: u8,
        gem: u32,
        mcash: u32,
        o2cash: u32,
        level: u32,
        win: u32,
        lose: u32,
        draw: u32,
        play_count: u32,
        experience: u32,
        is_admin: bool,
        equipment: Equipment,
        inventory: [ItemId; 30],
        padding: [u32; 5],
        effects: Vec<EffectEntry>,
    }

    let response = {
        let lists = crate::gateway::GET_ITEM_LIST();

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
            o2cash: user.info.o2gems,
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
#[derive(Copy, Clone, encoder::StructSerializer)]
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

#[gateway_derive::route(RequestId::ListRoomJoinRoom)]
async fn handle_join_room(client: &mut super::Client, packet: &mut super::Packet) {
    let Some(channel) = client.channel() else {
        println!("Client {} is not in a channel, cannot join room", client.id);
        return;
    };

    let Some(user) = client.user() else {
        println!("Client {} is not logged in, cannot join room", client.id);
        return;
    };

    match super::parse_request::<JoinRoomRequest>(&packet.body) {
        Ok(request) => {
            let result = {
                let mut channel = channel.lock().await;
                let password = request.password.to_string_lossy().to_string();

                channel.join_room(user, request.room_id, password).await
            };

            match result {
                Ok((room, slot, team)) => {
                    #[derive(encoder::StructSerializer)]
                    pub struct MemberInfo {
                        nickname: std::ffi::CString,
                        level: u32,
                        gender: u8,
                        is_room_master: bool,
                        color: TeamId,
                        ready: bool,
                        unk: u8,
                        equipment: Equipment,
                        list: Vec<MusicIdEntry>,
                    }

                    #[derive(encoder::StructSerializer, Clone, Copy, PartialEq, Eq)]
                    pub enum PositionStatus {
                        Empty = 0,
                        Occupied = 1,
                        Locked = 2,
                    }

                    struct PositionInfo {
                        position: u8,
                        status: PositionStatus,
                        member: Option<MemberInfo>,
                    }

                    impl encoder::StructEncodeImpl for PositionInfo {
                        fn impl_encode(
                            &self,
                            writer: &mut impl std::io::Write,
                        ) -> std::io::Result<()> {
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
                    struct JoinRoomResponse {
                        result: JoinErrorCode,
                        slot: u8,
                        team: TeamId,
                        name: std::ffi::CString,
                        music_id: MusicId,
                        arena: RoomArena,
                        mode: RoomMode,
                        diffculty: RoomDifficulty,
                        speed: RoomSpeed,
                        user_count: u32,
                        slots: [PositionInfo; 7],
                        skills: Vec<SkillId>,
                        premium: u16,
                    }

                    let (response, modifier) = {
                        let room = room.lock().await;

                        let mut slots = [const {
                            PositionInfo {
                                position: 0,
                                status: PositionStatus::Empty,
                                member: None,
                            }
                        }; 7];

                        let mut index = 0;
                        let modifier = room.get_modifier_report();

                        for (i, user) in room.players.iter().enumerate() {
                            if i == slot as usize {
                                continue;
                            }

                            let slot = &mut slots[index];
                            slot.position = i as u8;

                            match user {
                                crate::gateway::room::UserSlot::Available => {
                                    slot.status = PositionStatus::Empty
                                }
                                crate::gateway::room::UserSlot::User {
                                    team,
                                    host,
                                    ready,
                                    user,
                                    ..
                                } => {
                                    slot.status = PositionStatus::Occupied;

                                    let nickname = std::ffi::CString::new(user.nickname())
                                        .unwrap_or_else(|_| {
                                            std::ffi::CString::new("InvalidNickname").unwrap()
                                        });

                                    let member_info = MemberInfo {
                                        nickname,
                                        level: user.level(),
                                        gender: 1,
                                        is_room_master: *host,
                                        color: *team,
                                        ready: *ready,
                                        unk: 0,
                                        equipment: user.equipment(),
                                        list: user.music_list(),
                                    };

                                    slot.member = Some(member_info);
                                }
                                crate::gateway::room::UserSlot::Locked => {
                                    slot.status = PositionStatus::Locked
                                }
                            }

                            index += 1;
                        }

                        (
                            JoinRoomResponse {
                                result: JoinErrorCode::Success,
                                slot: slot as u8,
                                team: team,
                                name: std::ffi::CString::new(room.title.clone()).unwrap_or_default(),
                                music_id: room.music_id,
                                arena: room.arena,
                                mode: room.mode,
                                diffculty: room.difficulty,
                                speed: room.speed,
                                user_count: 7,
                                slots,
                                skills: room.skill_slot.clone(),
                                premium: 0,
                            },
                            modifier
                        )
                    };

                    client.room = Some(Arc::downgrade(&room));

                    crate::gateway::print_data(&crate::gateway::dump_data(ResponseId::ListRoomJoinRoom, &response));

                    client
                        .send_packet(ResponseId::ListRoomJoinRoom, &response)
                        .await
                        .expect("Failed to send join room response");

                    client
                        .send_packet(EventId::RoomOnAllModifiersChanged, &modifier)
                        .await
                        .expect("Failed to send modifier report response");
                }
                Err(JoinError::RoomNotFound) => {
                    println!(
                        "Client {} tried to join non existing room {}",
                        client.id, request.room_id
                    );
                    client
                        .send_packet(ResponseId::ListRoomJoinRoom, &JoinErrorCode::GenericError)
                        .await
                        .expect("Failed to send join room response");
                }
                Err(JoinError::IncorrectPassword) => {
                    println!(
                        "Client {} tried to join room {} with incorrect password",
                        client.id, request.room_id
                    );
                    client
                        .send_packet(
                            ResponseId::ListRoomJoinRoom,
                            &JoinErrorCode::InvalidPassword,
                        )
                        .await
                        .expect("Failed to send join room response");
                }
                Err(JoinError::RoomFull) => {
                    println!(
                        "Client {} tried to join full room {}",
                        client.id, request.room_id
                    );
                    client
                        .send_packet(ResponseId::ListRoomJoinRoom, &JoinErrorCode::RoomFull)
                        .await
                        .expect("Failed to send join room response");
                }
            }
        }
        Err(e) => {
            println!("Failed to parse join room request: {}", e);
        }
    }
}

#[derive(Clone, Copy, encoder::StructSerializer)]
pub enum LeaveErrorCode {
    Success = 0,
}

#[gateway_derive::route(RequestId::ListRoomLeaveRoom)]
async fn handle_leave_room(client: &mut super::Client, _packet: &mut super::Packet) {
    if let Some(channel) = client.channel() {
        let mut channel = channel.lock().await;

        if let Some(room) = client.room() {
            let id = room.lock().await.id;

            let Some(user) = client.user() else {
                println!("Client {} is not logged in, cannot leave room", client.id);
                return;
            };

            if let Err(e) = channel.leave_room(&user, id).await {
                println!(
                    "[Error] Failed to leave room {} for client {}: {:?}",
                    id, client.id, e
                );
            }
        }
    }

    client.room = None;
    client.send_packet(ResponseId::ListRoomLeaveRoom, &LeaveErrorCode::Success)
        .await
        .expect("Failed to send leave room response");

    println!("Client {} left the room", client.id);
}
