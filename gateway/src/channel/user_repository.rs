use std::sync::{Arc, atomic::AtomicUsize};

use crate::{
    channel::channel::INVALID_ROOM_ID,
    gateway::{commands::EventId, events::IEventData},
    user::User,
};

pub struct UserRepository {
    pub users: Vec<UserEntry>,
    pub count: Arc<AtomicUsize>,
}

impl UserRepository {
    pub fn new(count: Arc<AtomicUsize>) -> Self {
        UserRepository {
            users: Vec::new(),
            count,
        }
    }

    pub fn add(&mut self, user: &User) -> bool {
        if self.users.iter().any(|entry| entry.user.id == user.id) {
            return false;
        }

        self.users.push(UserEntry::new(user.clone()));
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        true
    }

    pub fn remove(&mut self, user_id: u64) -> bool {
        if let Some(pos) = self.users.iter().position(|entry| entry.user.id == user_id) {
            self.users.remove(pos);
            self.count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

            true
        } else {
            false
        }
    }

    pub fn get(&self, user_id: u64) -> Option<&UserEntry> {
        self.users.iter().find(|entry| entry.user.id == user_id)
    }

    pub fn get_mut(&mut self, user_id: u64) -> Option<&mut UserEntry> {
        self.users.iter_mut().find(|entry| entry.user.id == user_id)
    }

    pub fn broadcast<T: IEventData + 'static>(
        &mut self,
        id: EventId,
        event: T,
        exception: Option<&User>,
    ) {
        let boxed = Arc::new(event);

        for user in &self.users {
            if user.room_id != INVALID_ROOM_ID {
                continue;
            }

            if let Some(exc) = exception {
                if user.user == *exc {
                    continue;
                }
            }

            let _ = user.user.get_sender().send((id, boxed.clone()));
        }
    }

    pub fn broadcast_event(
        &mut self,
        id: EventId,
        event: Arc<dyn IEventData>,
        exception: Option<u64>,
    ) {
        for user in &self.users {
            if user.room_id != INVALID_ROOM_ID {
                continue;
            }

            if let Some(exc) = exception {
                if user.user.id == exc {
                    continue;
                }
            }

            let _ = user.user.get_sender().send((id, event.clone()));
        }
    }
}

pub struct UserEntry {
    pub user: User,
    pub room_id: u32,
}

impl UserEntry {
    pub fn new(user: User) -> Self {
        UserEntry {
            user,
            room_id: INVALID_ROOM_ID,
        }
    }

    pub fn is_in_room(&self) -> bool {
        self.room_id != INVALID_ROOM_ID
    }
}
