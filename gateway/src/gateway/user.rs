use database::{Equipment, UserInfo};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::gateway::{
    commands::EventId,
    events::IEventData,
    room::{MusicIdEntry, SkillId},
};

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub sender: Option<Arc<UnboundedSender<(EventId, Arc<dyn IEventData>)>>>,

    pub info: UserInfo,
    pub inventory: [ItemId; 30],
    pub equipment: Equipment,

    pub music_list: Vec<MusicIdEntry>,
}

impl User {
    pub async fn verify_credentials(
        username: &str,
        password: &str,
    ) -> Result<u64, UserError> {
        let pool = crate::gateway::GET_DATABASE().await;

        let Some(user) = pool.get_user_authentication_info(username).await else {
            return Err(UserError::InvalidCredentials);
        };

        if !bcrypt::verify(password, &user.password_hash).unwrap_or(false) {
            return Err(UserError::InvalidCredentials);
        }

        Ok(user.id as u64)
    }

    pub async fn try_create_session(
        user_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let pool = crate::gateway::GET_DATABASE().await;

        pool.create_session(user_id).await.map_err(|e| e.into())
    }

    pub async fn delete_session(user_id: u64) {
        let database = crate::gateway::GET_DATABASE().await;

        if let Err(e) = database.delete_session(user_id).await {
            println!(
                "[Error] Failed to delete session for user {}: {}",
                user_id, e
            );
        }
    }

    pub async fn request_user(
        id: u64,
        request_inventory: bool,
    ) -> Result<Self, UserError> {
        let pool = crate::gateway::GET_DATABASE().await;

        let Some(user) = pool.get_user_by_id(id).await else {
            return Err(UserError::Error(format!("User with id {} not found", id)));
        };

        let mut user = User {
            id: user.id as u64,
            sender: None,
            info: user,
            inventory: [ItemId::default(); 30],
            equipment: Equipment::default(),
            music_list: Vec::new(),
        };

        if request_inventory {
            let inventory = pool.get_inventory(user.id).await;
            for (i, item) in inventory.into_iter().enumerate() {
                if i >= user.inventory.len() {
                    break;
                }

                user.inventory[i] = ItemId {
                    id: item.item_id,
                    amount: item.quantity,
                };
            }

            let Some(equipment) = pool.get_equipment(user.id as u32).await else {
                return Ok(user);
            };

            user.equipment = equipment;
        }

        Ok(user)
    }

    pub async fn save(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = crate::gateway::GET_DATABASE().await;

        pool.save_user(&[&self.info]).await
    }

    pub async fn sync(&mut self) {
        let pool = crate::gateway::GET_DATABASE().await;

        let Some(user) = pool.get_user_by_id(self.id).await else {
            println!("Failed to sync user {}: not found in database", self.id);
            return;
        };

        self.info = user;
    }

    pub fn set_sender(&mut self, sender: Arc<UnboundedSender<(EventId, Arc<dyn IEventData>)>>) {
        self.sender = Some(sender);
    }

    pub fn set_music_list(&mut self, music_list: Vec<MusicIdEntry>) {
        self.music_list = music_list;
    }

    pub fn get_sender(&self) -> &Arc<UnboundedSender<(EventId, Arc<dyn IEventData>)>> {
        self.sender.as_ref().expect("Sender not set for user")
    }

    pub fn equipment(&self) -> Equipment {
        self.equipment
    }

    pub fn inventory(&self) -> [ItemId; 30] {
        self.inventory
    }

    pub fn skills(&self) -> Vec<SkillId> {
        panic!("Not implemented yet");
    }

    pub fn music_list(&self) -> Vec<MusicIdEntry> {
        self.music_list.clone()
    }

    pub fn username(&self) -> &str {
        &self.info.name
    }

    pub fn nickname(&self) -> &str {
        &self.info.nickname
    }

    pub fn level(&self) -> u32 {
        let base_exp = 1000.0;
        let growth_factor = 1.5;

        if self.info.exp < base_exp as u64 {
            0
        } else {
            let level = ((self.info.exp as f64 / base_exp).log(growth_factor) + 1.0).floor();
            level as u32
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for User {}

pub enum UserError {
    InvalidCredentials,
    Error(String),
}

#[derive(Debug, Clone, Copy, encoder::StructSerializer, Default)]
pub enum ItemInfo {
    #[default]
    Instrument = 0,
    Hair = 1,
    Accessory = 2,
    Glove = 3,
    Necklace = 4,
    Cloth = 5,
    Pant = 6,
    Glasses = 7,
    Earring = 8,
    ClothAccessory = 9,
    Shoes = 10,
    Face = 11,
    Wing = 12,
    InstrumentAccessory = 13,
    Pet = 14,
    HairAccessory = 15,
}

#[derive(Debug, Clone, Copy, encoder::StructSerializer, Default)]
pub struct ItemId {
    pub id: u32,
    pub amount: u32,
}
