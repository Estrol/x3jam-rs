use std::{collections::HashMap, sync::OnceLock};

static COMMAND_LINES: OnceLock<HashMap<String, Option<String>>> = OnceLock::new();

pub fn init() {
    let args = std::env::args().collect::<Vec<String>>();

    let mut config_map: HashMap<String, Option<String>> = HashMap::new();

    // The value based on next argument, if it exists and does not start with a dash

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") || arg.starts_with("-") {
            let key = arg.trim_start_matches('-').to_string();
            let value = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                i += 1;
                Some(args[i].clone())
            } else {
                None
            };
            config_map.insert(key, value);
        }
        i += 1;
    }

    COMMAND_LINES
        .set(config_map)
        .expect("Failed to set command line arguments");
}

pub fn get_str(key: &str, default: &str) -> String {
    let config = COMMAND_LINES
        .get()
        .expect("Command line arguments not initialized");

    if let Some(value) = config.get(key) {
        if let Some(value_str) = value {
            return value_str.clone();
        }
    }

    default.to_string()
}

pub fn get<T: Copy>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    let config = COMMAND_LINES
        .get()
        .expect("Command line arguments not initialized");

    if let Some(value) = config.get(key) {
        if let Some(value_str) = value {
            if let Ok(parsed_value) = value_str.parse::<T>() {
                return parsed_value;
            }
        }
    }

    default
}

pub fn contains(key: &str) -> bool {
    let config = COMMAND_LINES
        .get()
        .expect("Command line arguments not initialized");
    config.contains_key(key)
}
