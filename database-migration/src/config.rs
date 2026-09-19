use std::{collections::HashMap, sync::OnceLock};

static DATABASE: OnceLock<HashMap<String, HashMap<String, Option<String>>>> = OnceLock::new();

pub fn init() {
    let Ok(config) = ini::ini!(safe "./resources/config.ini") else {
        panic!("Failed to load configuration file: ./resources/config.ini");
    };

    DATABASE.set(config).expect("Failed to set configuration");
}

pub fn get_str(section: &str, key: &str, default: &str) -> String {
    let config = DATABASE.get().expect("Configuration not initialized");

    let section_lower = section.to_lowercase();
    let key_lower = key.to_lowercase();

    if let Some(section_map) = config.get(&section_lower) {
        if let Some(value) = section_map.get(&key_lower) {
            if let Some(value_str) = value {
                return value_str.clone();
            }
        }
    }

    default.to_string()
}

pub fn get<T: Copy>(section: &str, key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    let config = DATABASE.get().expect("Configuration not initialized");

    let section_lower = section.to_lowercase();
    let key_lower = key.to_lowercase();

    if let Some(section_map) = config.get(&section_lower) {
        if let Some(value) = section_map.get(&key_lower) {
            if let Some(value_str) = value {
                if let Ok(parsed_value) = value_str.parse::<T>() {
                    return parsed_value;
                }
            }
        }
    }

    default
}
