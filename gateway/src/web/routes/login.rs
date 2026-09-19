
macro_rules! make_response {
    ($status:expr, $body:expr) => {{
        let code = match $status {
            true => axum::http::StatusCode::OK,
            false => axum::http::StatusCode::BAD_REQUEST,
        };

        let result = LoginResponse {
            success: $status,
            message: Some($body.to_string()),
            token: None,
        };

        super::make_response_json!(code, result)
    }};
    ($status:expr, $body:expr, $token:expr) => {{
        let code = match $status {
            true => axum::http::StatusCode::OK,
            false => axum::http::StatusCode::BAD_REQUEST,
        };

        let result = LoginResponse {
            success: $status,
            message: Some($body.to_string()),
            token: Some($token),
        };

        super::make_response_json!(code, result)
    }};
}

pub async fn post(
    req: axum::http::Request<axum::body::Body>,
) -> impl axum::response::IntoResponse {
    let res = super::parse_request!(req, LoginRequest);

    if res.username.is_empty() || res.password.is_empty() {
        return make_response!(false, "Username and password cannot be empty");
    }

    let db = crate::database::get();

    let Some(user) = db.get_user_by_name(&res.username).await else {
        return make_response!(false, "Invalid username or password");
    };

    if !bcrypt::verify(&res.password, &user.password_hash).unwrap_or(false) {
        return make_response!(false, "Invalid username or password");
    }

    let Ok(Some(token)) = db.create_session(user.id).await else {
        return make_response!(false, "Failed to create session");
    };

    let servers = [
        "%s:16010",
        "%s:16010",
        "%s:16010",
        "%s:16010",
        "%s:16010",
        "%s:16010",
        "%s:16010",
        "%s:16010",
    ];

    const URLS: [&str; 14] = [
        const_format::formatcp!("http://%s/{}?", super::GAMEBOARD_FREE_LIST),
        const_format::formatcp!("http://%s/{}?", super::GAMEBOARD_JAMJJANG),
        const_format::formatcp!("http://%s/{}?", super::GAMEMUSIC),
        const_format::formatcp!("http://%s/{}?", super::GAMEBOARD_SCREEN_LIST),
        const_format::formatcp!("http://%s/{}?", super::GAMEBOARD_NOTE_LIST),
        const_format::formatcp!("http://%s/{}?", super::GAMEBOARD_NEMO_LIST),
        const_format::formatcp!("http://%s/{}", super::VIP_INDEX),
        const_format::formatcp!("http://%s/{}", super::GAMEMUSIC),
        const_format::formatcp!("http://%s/{}?", super::MUSICCAPSULE),
        const_format::formatcp!("http://%s/{}?", super::GIFT_INPUT),
        const_format::formatcp!("http://%s/{}?", super::PAYMENT_INPUT),
        const_format::formatcp!("http://%s/{}", super::PLANET_LIST),
        const_format::formatcp!("http://%s/{}", super::CHAT_ANNOUNCEMENT),
        const_format::formatcp!("http://%s/{}?myid=", super::GAMEFIND_MAIN),
    ];

    let token = TokenResponse {
        token,
        servers: servers.map(|s| s.to_string()),
        urls: URLS.map(|s| s.to_string()),
    };

    make_response!(true, "Login successful", token)
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: Option<String>,
    pub token: Option<TokenResponse>,
}

#[derive(serde::Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub servers: [String; 8],
    pub urls: [String; 14],
}