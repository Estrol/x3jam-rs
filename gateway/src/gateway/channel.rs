use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use tokio::sync::Mutex;

use crate::gateway::{
    commands::EventId,
    events::{IEventData, listroom::{ListRoomAddRoomEventArgs, ListRoomChangeRoomMaxPlayerEventArgs, ListRoomRemoveRoomEventArgs}},
    room::{Room, RoomMode, TeamId},
    routes::listroom::RoomEntry,
    user::User,
};

#[derive(Debug)]
pub struct Handle<T>(pub Arc<Mutex<T>>, pub u64);
pub struct ChannelHandle(pub Arc<Mutex<Channel>>, pub u32, pub u32); // channel id, server id

#[derive(Debug)]
pub struct UserHandle {
    pub user: User,
    pub onroom: bool,
}

// Hardcoded due to limits on client's recv buffer.
const MAX_ROOMS: usize = 100;

pub struct Channel {
    pub weak: Weak<Mutex<Channel>>,
    pub server_id: u32,
    pub channel_id: u32,

    pub list: Vec<super::ojnlist::Header>,
    pub users: Vec<UserHandle>,
    pub rooms: HashMap<u32, Arc<Mutex<super::room::Room>>>,
    pub max_users: usize,
    pub max_rooms: usize,
}

impl Channel {
    pub async fn new(
        server_id: u32,
        channel_id: u32,
        file: &str,
    ) -> Result<Arc<Mutex<Self>>, Box<dyn std::error::Error>> {
        let list = super::ojnlist::load_ojn_list(file).await?;

        println!(
            "Channel {}-{} loaded with {} songs",
            server_id,
            channel_id,
            list.len()
        );

        // Each room can hold up to 8 players, so the maximum number of users is MAX_ROOMS * 8
        let max_users = MAX_ROOMS * 8;

        let channel = Arc::new_cyclic(|weak| {
            Mutex::new(Self {
                weak: weak.clone(),
                server_id,
                channel_id,

                list,
                users: Vec::new(),
                rooms: HashMap::new(),
                max_users,
                max_rooms: MAX_ROOMS,
            })
        });

        Ok(channel)
    }

    pub async fn add_user(&mut self, user: &User) -> bool {
        if self.users.len() >= self.max_users {
            return false;
        }

        if self.users.iter().any(|u| *user == u.user) {
            return false;
        }

        self.users.push(UserHandle {
            user: user.clone(),
            onroom: false,
        });

        true
    }

    pub async fn remove_user(&mut self, user: &User) -> bool {
        if let Some(pos) = self.users.iter().position(|u| *user == u.user) {
            self.users.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn get_users(&self) -> Vec<super::routes::listroom::UserInfoEntry> {
        self.users
            .iter()
            .map(|user_handle| super::routes::listroom::UserInfoEntry {
                username: to_cstring(&user_handle.user.info.name),
                nickname: to_cstring(&user_handle.user.info.nickname),
                level: user_handle.user.level(),
            })
            .collect()
    }

    pub async fn get_rooms(&self) -> Vec<super::routes::listroom::RoomEntry> {
        let mut room_list = Vec::with_capacity(self.rooms.len());

        for i in 0..self.max_rooms as u32 {
            let mut room = RoomEntry::default();
            room.id = i;

            if let Some(handle) = self.rooms.get(&i) {
                let room_lock = handle.lock().await;

                room.state = room_lock.status;
                room.name = to_cstring(&room_lock.get_name_rate());
                room.ojn_id = room_lock.music_id;
                room.is_password = room_lock.password.is_some();
                room.difficulty = room_lock.difficulty;
                room.mode = room_lock.mode;
                room.speed = room_lock.speed;
                room.max_players = room_lock.max_players() as u8;
                room.current_players = room_lock.player_count() as u8;
                room.min_level = room_lock.min_level;
                room.max_level = room_lock.max_level;
                room.skills = room_lock.skill_slot.clone();
            }

            room_list.push(room);
        }

        room_list
    }

    pub async fn get_room(&self, room_id: u32) -> Option<Arc<Mutex<super::room::Room>>> {
        self.rooms
            .contains_key(&room_id)
            .then(|| Arc::clone(&self.rooms[&room_id]))
    }

    pub fn get_music_list(&self) -> Vec<super::routes::listroom::ServerMusicEntry> {
        self.list
            .iter() // limit by 800
            .map(|header| super::routes::listroom::ServerMusicEntry {
                songid: header.songid as u16,
                note_count_easy: header.note_count[0] as u16,
                note_count_normal: header.note_count[1] as u16,
                note_count_hard: header.note_count[2] as u16,
                price: 0,
            })
            .collect()
    }

    pub fn find_empty_room_id(&self) -> Option<u32> {
        for id in 0..self.max_rooms as u32 {
            if !self.rooms.contains_key(&id) {
                return Some(id);
            }
        }

        None
    }

    pub fn find_user_by_id(&mut self, user_id: u64) -> Option<&mut UserHandle> {
        self.users.iter_mut().find(|u| u.user.id == user_id)
    }

    pub async fn create_room(
        &mut self,
        user: &User,
        title: String,
        password: Option<String>,
        mode: RoomMode,
        min_level: u8,
        max_level: u8,
    ) -> Result<(u32, Arc<Mutex<super::room::Room>>), CreateError> {
        let Some(id) = self.find_empty_room_id() else {
            return Err(CreateError::ChannelFull);
        };

        let room = Room::new(
            self.weak.clone(),
            user,
            id,
            title.clone(),
            password,
            mode,
            min_level,
            max_level,
        );

        self.broadcast(
            EventId::ListRoomOnAddRoom,
            ListRoomAddRoomEventArgs {
                id,
                title: to_cstring(&room.title),
                mode,
                has_password: room.password.is_some(),
                min_level,
                max_level,
                premium: 0,
            },
            Some(user),
        );

        self.broadcast(
            EventId::ListRoomOnRoomPlayerCountChanged,
            ListRoomChangeRoomMaxPlayerEventArgs {
                id,
                max_player: room.max_players() as u8,
                current_player: room.player_count() as u8,
                premium: 0,
            },
            Some(user),
        );

        if let Some(user) = self.find_user_by_id(user.id) {
            user.onroom = true;
        }

        let room = Arc::new(Mutex::new(room));
        self.rooms.insert(id, Arc::clone(&room));

        Ok((id, room))
    }

    pub async fn join_room(
        &mut self,
        user: &super::user::User,
        room_id: u32,
        password: String,
    ) -> Result<(Arc<Mutex<super::room::Room>>, u8, TeamId), JoinError> {
        let (room_handle, slot, team, max_player, current_player) = {
            let user_handle = self
                .users
                .iter_mut()
                .find(|u| u.user.id == user.id)
                .ok_or(JoinError::RoomNotFound)?;

            let room_handle = self.rooms.get(&room_id).ok_or(JoinError::RoomNotFound)?;

            let mut room_lock = room_handle.lock().await;

            if let Some(room_password) = &room_lock.password {
                if *room_password != password {
                    return Err(JoinError::IncorrectPassword);
                }
            }

            if room_lock.player_count() >= room_lock.max_players() {
                return Err(JoinError::RoomFull);
            }

            let Some((slot, team)) = room_lock.add_player(user) else {
                return Err(JoinError::RoomFull);
            };

            // Update the user's status in the Channel to prevent further broadcasts
            user_handle.onroom = true;

            (Arc::clone(room_handle), slot, team, room_lock.max_players() as u8, room_lock.player_count() as u8)
        };

        self.broadcast(EventId::ListRoomOnRoomPlayerCountChanged, ListRoomChangeRoomMaxPlayerEventArgs {
            id: room_id,
            max_player: max_player as u8,
            current_player: current_player as u8,
            premium: 0,
        }, Some(&user));

        Ok((room_handle, slot, team))
    }

    pub async fn leave_room(&mut self, user: &super::user::User, room_id: u32) -> Result<(), LeaveError> {
        let (player_count, max_player) = {
            let room_handle = self.rooms.get(&room_id).ok_or(LeaveError::RoomNotFound)?;

            let mut room_lock = room_handle.lock().await;

            match room_lock.remove_client(user).await {
                Some(count) => (count, room_lock.max_players() as u8),
                None => return Err(LeaveError::NotInRoom),
            }
        };

        if let Some(user) = self.find_user_by_id(user.id) {
            user.onroom = false;
        }

        if player_count == 0 {
            self.rooms.remove(&room_id);

            self.broadcast(
                EventId::ListRoomOnRemoveRoom,
                ListRoomRemoveRoomEventArgs { id: room_id },
                Some(&user)
            );
        } else {
            self.broadcast(
                EventId::ListRoomOnRoomPlayerCountChanged,
                ListRoomChangeRoomMaxPlayerEventArgs {
                    id: room_id,
                    max_player,
                    current_player: player_count as u8,
                    premium: 0,
                },
                Some(&user),
            );
        }

        Ok(())
    }

    pub fn broadcast<T: IEventData + 'static>(
        &mut self,
        id: EventId,
        event: T,
        exception: Option<&super::user::User>,
    ) {
        let boxed = Arc::new(event);

        for user in &self.users {
            if user.onroom {
                continue;
            }

            if let Some(exc) = exception {
                if user.user == *exc {
                    continue;
                }
            }

            let _ = user
                .user
                .get_sender()
                .send((id, boxed.clone()));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CreateError {
    ChannelFull
}

#[derive(Debug, Clone, Copy)]
pub enum JoinError {
    RoomNotFound,
    IncorrectPassword,
    RoomFull,
}

#[derive(Debug, Clone, Copy)]
pub enum LeaveError {
    RoomNotFound,
    NotInRoom,
}

pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).expect("Failed to convert string to CString")
}
