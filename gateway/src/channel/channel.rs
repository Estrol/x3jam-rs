use std::{
    collections::HashMap, sync::{Arc, atomic::AtomicUsize}, time::Duration,
};

use crate::{
    channel::{
        ChannelWeakHandle, ojnlist::Header, user_repository::UserRepository,
    }, gateway::{
        commands::EventId, events::listroom::{ListRoomAddRoomEventArgs, ListRoomChangeRoomMaxPlayerEventArgs, ListRoomChatEventArgs, ListRoomRemoveRoomEventArgs}, routes::{listroom::{
            JoinErrorCode, JoinRoomResponse, RoomEntry,
            ServerMusicEntry, UserInfoEntry,
        }, room::CreateRoomResult},
    }, room::{ModifierReport, RoomCommand, RoomHandle, RoomMode, RoomStatus, RoomWeakHandle}, user::User,
};

pub const INVALID_ROOM_ID: u32 = 0;
pub const MAX_ROOM_ID: u32 = 120;

pub struct Room {
    pub handle: RoomHandle,
    pub token: tokio_util::sync::CancellationToken,
}

pub struct Channel {
    pub region: u32,
    pub id: u32,

    pub users: UserRepository,
    pub rooms: HashMap<u32, Room>,

    pub lists: Vec<Header>,
}

impl Channel {
    pub async fn new(region: u32, id: u32, path: &str) -> (Self, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));

        let rooms = HashMap::new();
        let users = UserRepository::new(counter.clone());

        let ojnlist = super::ojnlist::load_ojn_list(path)
            .await
            .expect("Failed to load OJN list");

        (
            Channel {
                region,
                id,
                users,
                rooms,
                lists: ojnlist,
            },
            counter,
        )
    }

    pub async fn connect(&mut self, user: &User) -> bool {
        self.users.add(user)
    }

    pub async fn disconnect(&mut self, user: u64) -> bool {
        let (id, is_in_room) = {
            let Some(user_entry) = self.users.get_mut(user) else {
                return false;
            };
            
            (user_entry.user.id, user_entry.is_in_room())
        };

        if is_in_room {
            self.leave_room(id).await;
        }

        self.users.remove(id)
    }

    pub fn get_server_list(&self) -> Vec<ServerMusicEntry> {
        self.lists
            .iter()
            .map(|header| ServerMusicEntry {
                songid: header.songid as u16,
                note_count_easy: header.note_count[0] as u16,
                note_count_normal: header.note_count[1] as u16,
                note_count_hard: header.note_count[2] as u16,
                price: 0,
            })
            .collect()
    }

    pub fn get_users(&self) -> Vec<UserInfoEntry> {
        self.users
            .users
            .iter()
            .map(|user_handle| UserInfoEntry {
                username: to_cstring(&user_handle.user.info.name),
                nickname: to_cstring(&user_handle.user.info.nickname),
                level: user_handle.user.level(),
            })
            .collect()
    }

    pub async fn get_rooms(&self) -> Vec<RoomEntry> {
        let futures = (0..MAX_ROOM_ID).map(|i| async move {
            let mut room = RoomEntry::default();
            room.id = i;

            if let Some(handle) = self.rooms.get(&i) {
                if let Ok(data) = handle.handle.send_timed::<RoomEntry>(RoomCommand::GetEntryInfo, Duration::from_secs(5)).await {
                    room = data;
                } else {
                    room.state = RoomStatus::Playing;
                    room.name = to_cstring(&format!("Room {} error", i));
                }
            }

            room
        });

        futures::future::join_all(futures).await
    }

    pub fn find_empty_room_id(&self) -> Option<u32> {
        for room_id in 0..MAX_ROOM_ID {
            if !self.rooms.contains_key(&room_id) {
                return Some(room_id);
            }
        }
        None
    }

    pub async fn create_room(
        &mut self,
        channel_handle: &ChannelWeakHandle,
        user_id: u64,
        title: String,
        password: Option<String>,
        mode: RoomMode,
        min_level: u8,
        max_level: u8,
    ) -> (CreateRoomResult, Option<RoomWeakHandle>) {
        let Some(room_id) = self.find_empty_room_id() else {
            return (CreateRoomResult::Full, None);
        };

        let title_cloned = title.clone();
        let has_password = password.is_some();

        let handle = {
            let Some(user_entry) = self.users.get_mut(user_id) else {
                return (CreateRoomResult::Full, None);
            };

            if user_entry.is_in_room() {
                return (CreateRoomResult::Full, None);
            }

            let token = tokio_util::sync::CancellationToken::new();

            let Ok(room) = crate::room::make_room(
                &user_entry.user,
                channel_handle.clone(),
                token.clone(),
                room_id,
                title.clone(),
                password.clone(),
                mode,
                min_level,
                max_level,
            ).await else {
                return (CreateRoomResult::Full, None);
            };

            let weak = room.make_weak();

            user_entry.room_id = room_id;
            self.rooms.insert(room_id, Room { handle: room, token });

            weak
        };

        self.users.broadcast(
            EventId::ListRoomOnAddRoom,
            ListRoomAddRoomEventArgs {
                id: handle.id,
                title: to_cstring(&title_cloned),
                mode,
                has_password,
                min_level,
                max_level,
                premium: 0,
            },
            None,
        );

        self.users.broadcast(
            EventId::ListRoomOnRoomPlayerCountChanged,
            ListRoomChangeRoomMaxPlayerEventArgs {
                id: handle.id,
                max_player: 8 as u8, // By default it's 8
                current_player: 1 as u8, // The creator is the first player
                premium: 0,
            },
            None
        );

        (CreateRoomResult::Success, Some(handle))
    }

    pub async fn join_room(
        &mut self,
        room_id: u32,
        user_id: u64,
        password: Option<String>,
    ) -> (JoinRoomResponse, Option<(RoomWeakHandle, ModifierReport)>) {
        let Some(room) = self.rooms.get_mut(&room_id) else {
            return (JoinRoomResponse::room_not_found(), None);
        };

        let Some(user) = self.users.get_mut(user_id) else {
            return (JoinRoomResponse::user_not_found(), None);
        };

        let Ok(result) = room.handle.send::<(JoinRoomResponse, Option<ModifierReport>)>(RoomCommand::JoinRoom { user: user.user.clone(), password }).await else {
            return (JoinRoomResponse::room_full(), None);
        };

        if result.0.result == JoinErrorCode::Success {
            user.room_id = room_id;
        }

        (result.0, result.1.map(|modifier| (room.handle.make_weak(), modifier)))
    }

    pub async fn leave_room(&mut self, user_id: u64) -> bool {
        let (room_id, player_count) = {
            let Some((user, Some(room))) = self.get_user_mut(user_id) else {
                return false;
            };

            let Ok(player_count) = room.handle.send::<Option<usize>>(RoomCommand::LeaveRoom { user_id: user.id }).await else {
                return false;
            };

            let Some(player_count) = player_count else {
                return false;
            };

            (room.handle.id, player_count)
        };

        if player_count == 0 {
            let room = self.rooms.remove(&room_id)
                .expect("Room should exist");

            room.token.cancel();
            room.handle.join_handle.await
                .expect("Room task should finish");

            self.users.broadcast(
                EventId::ListRoomOnRemoveRoom,
                ListRoomRemoveRoomEventArgs {
                    id: room_id,
                },
                None,
            );
        }

        let Some(user_entry) = self.users.get_mut(user_id) else {
            return false;
        };

        user_entry.room_id = INVALID_ROOM_ID;

        true
    }

    pub fn get_user(&self, user_id: u64) -> Option<(&User, Option<&Room>)> {
        if let Some(user_entry) = self.users.get(user_id) {
            let room = if user_entry.is_in_room() {
                self.rooms.get(&user_entry.room_id)
            } else {
                None
            };

            Some((&user_entry.user, room))
        } else {
            None
        }
    }

    pub fn get_user_mut(&mut self, user_id: u64) -> Option<(&mut User, Option<&mut Room>)> {
        if let Some(user_entry) = self.users.get_mut(user_id) {
            let room = if user_entry.is_in_room() {
                self.rooms.get_mut(&user_entry.room_id)
            } else {
                None
            };

            Some((&mut user_entry.user, room))
        } else {
            None
        }
    }

    pub fn chat(&mut self, user_id: u64, message: String) {
        let Some(user_entry) = self.users.get(user_id) else {
            return;
        };

        self.users.broadcast(
            EventId::ListRoomOnChat,
            ListRoomChatEventArgs {
                author: to_cstring(&user_entry.user.info.name),
                message: to_cstring(&message),
            },
            None,
        );
    }

    pub async fn shutdown(&mut self) {
        for (_, room) in self.rooms.iter() {
            room.token.cancel();
        }

        let disconnect_futures = self.rooms
            .iter_mut()
            .map(|(_, room)| &mut room.handle.join_handle)
            .collect::<Vec<_>>();

        let timeout = tokio::time::sleep(Duration::from_secs(5));

        tokio::select! {
            _ = futures::future::join_all(disconnect_futures) => {},
            _ = timeout => {
                println!("Timeout while waiting for rooms to shutdown");
            }
        }
    }
}

pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new("").unwrap())
}
