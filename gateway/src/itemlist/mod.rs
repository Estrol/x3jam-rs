use std::sync::OnceLock;

pub mod game_modifier;
pub mod item_gender;

pub use game_modifier::{GameModifier, GameModifierType};
pub use item_gender::ItemGender;

pub mod itemlist;

lazy_static::lazy_static! {
    static ref ITEM_LIST: OnceLock<itemlist::ItemList> = OnceLock::new();
}

pub async fn init() {
    let filename = super::config::get_str("ITEMLIST", "FILENAME", "itemlist.dat");
    let path = format!("./resources/data/{}", filename);

    let list = itemlist::ItemList::load_from_file(&path)
        .await
        .expect("Failed to load item list");

    ITEM_LIST.set(list).expect("Failed to set item list");
}

pub fn get() -> &'static itemlist::ItemList {
    ITEM_LIST.get().expect("Item list not initialized")
}
