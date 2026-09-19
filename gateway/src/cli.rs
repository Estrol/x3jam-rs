use crate::session::{LoginError, Session};

pub async fn handle(data: String) -> bool {
    let mut args: Vec<String> = data.split_whitespace().map(|s| s.to_string()).collect();

    // return false, program stop
    match args.get(0).map(|s| s.to_lowercase()) {
        Some(cmd) if cmd == "register" => {
            if args.len() < 3 {
                log::output!("[CLI] Usage: register <username> <password> <email> [gender:male|female]");
                return true;
            }

            let username = args.remove(1);
            let password = args.remove(1);
            let email = args.remove(1);

            let gender: Option<database::CharacterGender> = if args.len() > 1 {
                match args.remove(1).to_lowercase().as_str() {
                    "male" => Some(database::CharacterGender::Male),
                    "female" => Some(database::CharacterGender::Female),
                    _ => {
                        log::output!("[CLI] Invalid gender specified. Use 'male' or 'female'.");
                        return true;
                    }
                }
            } else {
                None
            };

            match crate::user::User::register(
                &username,
                &password,
                &username,
                &email,
                gender.unwrap_or(database::CharacterGender::Male)).await {
                Ok(success) => {
                    if success {
                        log::output!("[CLI] User '{}' registered successfully.", username);
                    } else {
                        log::output!("[CLI] User '{}' already exists.", username);
                    }
                }
                Err(_) => {
                    log::output!("[CLI] Error occurred while registering user '{}'.", username);
                }
            }

            return true;
        }

        Some(cmd) if cmd == "q" || cmd == "quit" || cmd == "exit" => {
            return false;
        }

        Some(cmd) if cmd == "login" => {
            if args.len() < 3 {
                log::output!("[CLI] Usage: login <username> <password>");
                return true;
            }

            let username = args.remove(1);
            let password = args.remove(1);

            let session = match Session::login(&username, &password).await {
                Ok(session) => session,
                Err(LoginError::InvalidCredentials) => {
                    log::output!("[CLI] Invalid username or password");
                    return true;
                }
                Err(LoginError::GenericError(e)) => {
                    log::output!("[CLI] Error logging in: {}", e);
                    return true;
                }
            };

            if let Err(e) = session.update().await {
                log::output!("[CLI] Failed to update session for user '{}': {}", username, e);
                return true;
            }

            log::output!(
                "[CLI] User '{}' logged in successfully. Session token: {}",
                username,
                session.token
            );

            return true;
        }

        Some(cmd) if cmd == "help" || cmd == "h" => {
            log::output!("[CLI] Available commands:");
            log::output!(
                "[CLI]   register <username> <password> <email> [gender:male|female] - Register a new user"
            );
            log::output!("[CLI]   quit | exit | q - Exit the program");
            log::output!("[CLI]   help | h - Show this help message");
            log::output!("[CLI]   login <username> <password> - Log in as a user");
            return true;
        }
        Some(cmd) => {
            log::output!("[CLI] Unknown command: {}", cmd);
            log::output!("[CLI] Type 'help' or 'h' for a list of available commands.");
            return true;
        }
        None => return true,
    }
}
