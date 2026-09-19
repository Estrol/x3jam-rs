use database::{Equipment, UserInfo};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub mod itemid;
pub mod itemtype;

pub use itemid::ItemId;
pub use itemtype::ItemType;

use crate::{
    gateway::{commands::EventId, events::IEventData},
    room::{MusicId, SkillId},
};

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub sender: Option<UnboundedSender<(EventId, Arc<dyn IEventData>)>>,

    pub info: UserInfo,
    pub inventory: [ItemId; 30],
    pub equipment: Equipment,

    pub music_list: Vec<MusicId>,
}

impl User {
    pub async fn register(
        username: &str,
        password: &str,
        nickname: &str,
        email: &str,
        gender: database::CharacterGender,
    ) -> Result<bool, UserError> {
        let db = crate::database::get();

        if db.get_user_by_name(username).await.is_some() {
            return Ok(false);
        }

        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| UserError::Error(format!("Failed to hash password: {}", e)))?;

        if db.create_user(username, &hash, nickname, email, gender).await.is_none() {
            return Ok(false);
        }

        Ok(true)
    }
    pub async fn request_user(id: u64, request_inventory: bool) -> Result<Self, UserError> {
        let pool = crate::database::get();

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
            for item in inventory {
                if item.slot as usize >= user.inventory.len() {
                    log::warn!(
                        "Inventory slot {} out of bounds for user {}",
                        item.slot,
                        user.id
                    );
                    continue;
                }

                user.inventory[item.slot as usize] = ItemId {
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

    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = crate::database::get();

        pool.save_user(&[&self.info]).await?;
        pool.save_equipment(self.id, &self.equipment).await?;

        let items_db = self
            .inventory
            .iter()
            .enumerate()
            .map(|(slot, item)| database::Item {
                slot: slot as u8,
                item_id: item.id,
                quantity: item.amount,
            })
            .collect::<Vec<_>>();

        pool.save_inventory(self.id, &items_db).await?;

        Ok(())
    }

    pub async fn sync(&mut self) {
        let pool = crate::database::get();

        let Some(user) = pool.get_user_by_id(self.id).await else {
            log::info!("Failed to sync user {}: not found in database", self.id);
            return;
        };

        let Some(equipment) = pool.get_equipment(self.id as u32).await else {
            log::info!(
                "Failed to sync user {}: equipment not found in database",
                self.id
            );
            return;
        };

        let inventory = pool.get_inventory(self.id).await;
        let mut inventory_array = [ItemId::default(); 30];
        for item in inventory {
            if item.slot as usize >= inventory_array.len() {
                log::warn!(
                    "Inventory slot {} out of bounds for user {}",
                    item.slot,
                    self.id
                );
                continue;
            }

            inventory_array[item.slot as usize] = ItemId {
                id: item.item_id,
                amount: item.quantity,
            };
        }

        self.info = user;
        self.equipment = equipment;
        self.inventory = inventory_array;
    }

    pub fn set_music_list(&mut self, music_list: Vec<MusicId>) {
        self.music_list = music_list;
    }

    pub fn get_sender(&self) -> &UnboundedSender<(EventId, Arc<dyn IEventData>)> {
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

    pub fn music_list(&self) -> Vec<MusicId> {
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

    pub fn get_item_from_slot(&self, slot: u32) -> Option<ItemId> {
        if slot >= self.inventory.len() as u32 {
            return None;
        }

        Some(self.inventory[slot as usize])
    }

    pub fn set_item_in_slot(&mut self, slot: u32, item: ItemId) {
        if slot >= self.inventory.len() as u32 {
            return;
        }

        self.inventory[slot as usize] = item;
    }

    pub fn set_equipment(&mut self, slot: u32, item_id: u32) -> Option<u32> {
        let old_item = match slot {
            0 => self.equipment.instrument,
            1 => self.equipment.hair,
            2 => self.equipment.accessory,
            3 => self.equipment.glove,
            4 => self.equipment.necklace,
            5 => self.equipment.cloth,
            6 => self.equipment.pant,
            7 => self.equipment.glass,
            8 => self.equipment.earring,
            9 => self.equipment.cloth_accessory,
            10 => self.equipment.shoes,
            11 => self.equipment.face,
            12 => self.equipment.wing,
            13 => self.equipment.instrument_accessory,
            14 => self.equipment.pet,
            15 => self.equipment.hair_accessory,
            _ => return None,
        };

        match slot {
            0 => self.equipment.instrument = item_id,
            1 => self.equipment.hair = item_id,
            2 => self.equipment.accessory = item_id,
            3 => self.equipment.glove = item_id,
            4 => self.equipment.necklace = item_id,
            5 => self.equipment.cloth = item_id,
            6 => self.equipment.pant = item_id,
            7 => self.equipment.glass = item_id,
            8 => self.equipment.earring = item_id,
            9 => self.equipment.cloth_accessory = item_id,
            10 => self.equipment.shoes = item_id,
            11 => self.equipment.face = item_id,
            12 => self.equipment.wing = item_id,
            13 => self.equipment.instrument_accessory = item_id,
            14 => self.equipment.pet = item_id,
            15 => self.equipment.hair_accessory = item_id,
            _ => return None,
        };

        log::info!(
            "Swapped equipment in slot {}: old item {}, new item {}",
            slot,
            old_item,
            item_id
        );

        Some(old_item)
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
