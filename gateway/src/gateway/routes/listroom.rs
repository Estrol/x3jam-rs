use database::Equipment;
use encoder::stringutil::CStrEx;

use crate::{
    channel::ChannelCommand,
    gateway::{commands::ResponseId, events::room::PlayingState},
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
#[derive(Debug, Clone)]
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

impl<T: encoder::StructDecodeImpl> encoder::StructDecodeImpl for VecU16<T> {
    fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
        use byteorder_lite::{LittleEndian, ReadBytesExt as _};

        let length = reader.read_u16::<LittleEndian>()?;
        let mut data = Vec::with_capacity(length as usize);

        for _ in 0..length {
            let item = T::impl_decode(reader)?;
            data.push(item);
        }

        Ok(Self { length, data })
    }
}

#[gateway_derive::route(RequestId::PlanetGetServerList)]
async fn handle_server_list(client: &mut super::Client, _packet: &()) {
    let Some((_, ch)) = client.channel() else {
        log::info!(
            "Client {} is not in a channel, cannot get server list",
            client.id
        );
        return;
    };

    let Ok(list) = ch
        .send::<Vec<ServerMusicEntry>>(ChannelCommand::RequestServerList)
        .await
    else {
        log::info!(
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
        log::info!(
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
        log::info!("Client {} is not in a channel, cannot sync info", client.id);
        return;
    };

    let Ok(user) = ch
        .send::<User>(ChannelCommand::SyncUserInfo { user_id })
        .await
    else {
        log::info!(
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
    pub username: CStrEx,
    pub nickname: CStrEx,
    pub level: u32,
}

#[derive(Debug, encoder::StructSerializer, Clone)]
pub struct RoomEntry {
    pub id: u32,
    pub state: RoomStatus,
    pub name: CStrEx,
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
            name: CStrEx::from_cstr(DEFAULT_TITLE),
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
        log::info!(
            "Client {} is not in a channel, cannot get room list",
            client.id
        );
        return;
    };

    let get_rooms_fut = channel.send::<Vec<RoomEntry>>(ChannelCommand::GetRooms);
    let get_users_fut = channel.send::<Vec<UserInfoEntry>>(ChannelCommand::GetUsers);

    let (Ok(rooms), Ok(users)) = tokio::join!(get_rooms_fut, get_users_fut) else {
        log::info!(
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

#[derive(Debug, Clone, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct EffectEntry {
    pub id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct GiftInfo {
    pub id: u32,
    pub item: u32,
    pub sender: CStrEx, // capped at 28 bytes.
}

#[derive(Debug, Clone, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct CharacterResponse {
    pub invalid: u32,
    pub nickname: CStrEx,
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
    pub item_inventory: [u32; 30],
    pub music_inventory: Vec<MusicId>,     // 4
    pub unreaded_messages: u32,            // 8
    pub item_gift_list: VecU16<GiftInfo>,  // 10
    pub music_gift_list: VecU16<GiftInfo>, // 12
    pub item_cash: u32,                    // 16
    pub music_cash: u32,                   // 20
    pub effects: Vec<EffectEntry>,
}

impl Default for CharacterResponse {
    fn default() -> Self {
        Self {
            invalid: 0,
            nickname: CStrEx::empty(),
            gender: 0,
            gem: 0,
            mcash: 0,
            o2cash: 0,
            level: 0,
            win: 0,
            lose: 0,
            draw: 0,
            play_count: 0,
            experience: 0,
            is_admin: false,
            equipment: Equipment::default(),
            item_inventory: [0; 30],
            music_inventory: Vec::new(),
            unreaded_messages: 0,
            item_gift_list: VecU16::new(Vec::new()),
            music_gift_list: VecU16::new(Vec::new()),
            item_cash: 0,
            music_cash: 0,
            effects: Vec::new(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use encoder::StructReadExt;

    use super::*;

    #[test]
    fn test() {
        let peer1_6: [u8; 425] = [
            /* Packet 30 */
            0x00, 0x00, 0x00, 0x00, 0x65, 0x73, 0x74, 0x72, 0x6f, 0x6c, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x98, 0x3a, 0x00, 0x00, 0x98, 0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x23, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x9c, 0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x9a, 0x00, 0x00, 0x00, 0x0f,
            0x27, 0x00, 0x00, 0x98, 0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x96, 0x00, 0x00,
            0x00, 0x0f, 0x27, 0x00, 0x00, 0x94, 0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x92,
            0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x9c,
            0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x9a, 0x00, 0x00, 0x00, 0x0f, 0x27, 0x00,
            0x00, 0x98, 0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x96, 0x00, 0x00, 0x00, 0x0f,
            0x27, 0x00, 0x00, 0x94, 0x00, 0x00, 0x00, 0x0f, 0x27, 0x00, 0x00, 0x92, 0x00, 0x00,
            0x00, 0x0f, 0x27, 0x00, 0x00,
        ];

        let peer1_5: [u8; 304] = [
            /* Packet 41 */
            0x00, 0x00, 0x00, 0x00, 0x44, 0x4d, 0x4a, 0x41, 0x4d, 0x00, 0x00, 0x0a, 0x00, 0x00,
            0x00, 0xa0, 0x86, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x71, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x9c, 0x00, 0x00, 0x00, 0x9a, 0x00, 0x00, 0x00, 0x98, 0x00, 0x00, 0x00, 0x96, 0x00,
            0x00, 0x00, 0x94, 0x00, 0x00, 0x00, 0x92, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x9c, 0x00, 0x00, 0x00, 0xa0, 0x86, 0x01, 0x00, 0x9a, 0x00,
            0x00, 0x00, 0xa0, 0x86, 0x01, 0x00, 0x98, 0x00, 0x00, 0x00, 0xa0, 0x86, 0x01, 0x00,
            0x96, 0x00, 0x00, 0x00, 0xa0, 0x86, 0x01, 0x00, 0x94, 0x00, 0x00, 0x00, 0xa0, 0x86,
            0x01, 0x00, 0x92, 0x00, 0x00, 0x00, 0xa0, 0x86, 0x01, 0x00,
        ];

        let mut character_response = CharacterResponse::default();

        let mut reader = std::io::Cursor::new(peer1_5);

        reader.read_struct(&mut character_response);

        dbg!(&character_response);
    }
}

#[gateway_derive::route(RequestId::ListRoomGetCharacter)]
async fn handle_get_character(client: &mut super::Client, _packet: &()) {
    let Some(user) = client.user() else {
        log::info!(
            "Client {} is not logged in, cannot get character info",
            client.id
        );
        return;
    };

    let response = {
        let lists = crate::itemlist::get();

        let mut effects = Vec::new();
        let mut inventory = [0u32; 30];

        for (i, item) in user.inventory.iter().enumerate() {
            let Some(item_info) = lists.get_item(item.id) else {
                continue;
            };

            if item_info.modifier_type != GameModifierType::None {
                effects.push(EffectEntry {
                    id: item.id,
                    amount: item.amount,
                });
            }

            inventory[i] = item.id;
        }

        CharacterResponse {
            invalid: 0,
            nickname: CStrEx::from_string(&user.nickname()),
            gender: match user.info.gender {
                database::CharacterGender::Female => 0,
                database::CharacterGender::Male => 1,
            },
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
            item_inventory: inventory,
            music_inventory: Vec::new(),
            unreaded_messages: 0,
            item_gift_list: VecU16::new(Vec::new()),
            music_gift_list: VecU16::new(Vec::new()),
            item_cash: 0,
            music_cash: 0,
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
    pub password: CStrEx,
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
    pub nickname: CStrEx,
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
    pub name: CStrEx,
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
            name: CStrEx::from_cstr(DEFAULT_ROOM_NAME),
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
        log::info!("Client {} is not in a channel, cannot join room", client.id);
        return;
    };

    let password_option = if packet.password.len() > 0 {
        Some(packet.password.to_string())
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
        log::info!(
            "Failed to join room {} for client {}",
            packet.room_id,
            client.id
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
        log::info!(
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
        log::info!("Failed to send leave room command for client {}", client.id);
        return;
    }

    client.room_handle = None;

    client
        .send_packet(ResponseId::ListRoomLeaveRoom, &LeaveErrorCode::Success)
        .await
        .expect("Failed to send leave room response");
}
