pub struct Session {
    pub uid: u64,
    pub token: String,
}

pub enum LoginError {
    InvalidCredentials,
    GenericError(Box<dyn std::error::Error + Send + Sync>),
}

impl Session {
    pub async fn login(username: &str, password: &str) -> Result<Self, LoginError> {
        let db = crate::database::get();

        let Some(user) = db.get_user_by_name(username).await else {
            return Err(LoginError::InvalidCredentials);
        };

        if !bcrypt::verify(password, &user.password_hash).unwrap_or(false) {
            return Err(LoginError::InvalidCredentials);
        }

        let token = match db.create_session(user.id).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                return Err(LoginError::GenericError(
                    "Failed to create session".into(),
                ));
            }
            Err(e) => {
                return Err(LoginError::GenericError(e));
            }
        };

        Ok(Session { uid: user.id, token })
    }

    pub async fn verify(token: &str) -> Result<Option<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let db = crate::database::get();

        let Some(session) = db.get_session_by_token(token).await? else {
            return Ok(None);
        };

        Ok(Some(Session {
            uid: session.user_id,
            token: session.token,
        }))
    }

    pub async fn bind_socket(&self, socket_id: u32) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let db = crate::database::get();

        db.session_set_socket(self.uid, socket_id).await
    }

    pub async fn unbind_socket(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let db = crate::database::get();

        db.session_remove_socket(self.uid).await
    }

    pub async fn update(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let db = crate::database::get();

        db.update_session(self.uid).await
    }
}