use std::sync::OnceLock;

static DATABASE: OnceLock<database::GameDatabase> = OnceLock::new();

fn parse_equipment_array(config_str: &str) -> [u32; 16] {
    let mut arr = [0; 16];
    for (i, val) in config_str
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .take(16)
        .enumerate()
    {
        arr[i] = val;
    }
    arr
}

fn parse_items(config_str: &str) -> Vec<(u32, u32)> {
    config_str
        .split('|')
        .filter_map(|s| {
            let clean_s = s.trim_matches(|c| c == '{' || c == '}');
            let (id_str, count_str) = clean_s.split_once(',')?;

            Some((id_str.parse().ok()?, count_str.parse().ok()?))
        })
        .collect()
}

pub async fn init() {
    let connection_string = super::config::get_str("DATABASE", "URL", "");
    if connection_string.is_empty() {
        panic!("Database connection string is empty. Please set DATABASE_URL in the config.");
    }

    log::info!("Connectin to database...");

    let mut db = database::GameDatabase::new(&connection_string).await;

    db.set_default_mcash(super::config::get::<u32>("USER", "MCash", 0));
    db.set_default_gold(super::config::get::<u32>("USER", "Gold", 0));

    let default_character_f = super::config::get_str("USER", "FCharacters", "");
    let default_character_m = super::config::get_str("USER", "MCharacters", "");

    db.set_default_equipment(
        database::CharacterGender::Female,
        parse_equipment_array(&default_character_f),
    );
    db.set_default_equipment(
        database::CharacterGender::Male,
        parse_equipment_array(&default_character_m),
    );

    let default_items_str = super::config::get_str("USER", "Items", "");
    db.set_default_items(parse_items(&default_items_str));
    
    DATABASE.set(db).expect("Failed to set database");
}

pub async fn close() {
    if let Some(db) = DATABASE.get() {
        db.close().await;
    }
}

pub fn get() -> &'static database::GameDatabase {
    DATABASE.get().expect("Database not initialized")
}
