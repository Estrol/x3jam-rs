use axum::{
    Router, body::to_bytes, response::IntoResponse as _, routing::{get, post},
};

pub mod home;
pub mod chat_announcement;
pub mod getusers;
pub mod payment;
pub mod planetlist;
pub mod userlist;
pub mod registration;
pub mod login;
pub mod noop;

const GAMEBOARD_FREE_LIST: &str = "gameboard_free_list";
const GAMEBOARD_JAMJJANG: &str = "gameboard_jamjjang";
const GAMEBOARD_SCREEN_LIST: &str = "gameboard_screen_list";
const GAMEBOARD_NOTE_LIST: &str = "gameboard_note_list";
const GAMEBOARD_NEMO_LIST: &str = "gameboard_memo_list";
const VIP_INDEX: &str = "vip_index";
const GAMEMUSIC: &str = "gamemusic";
const MUSICCAPSULE: &str = "musiccapsule";
const GIFT_INPUT: &str = "gift_input";
const PAYMENT_INPUT: &str = "payment_input";
const PLANET_LIST: &str = "planet_list";
const CHAT_ANNOUNCEMENT: &str = "chat_announcement";
const GAMEFIND_MAIN: &str = "gamefind_main";

pub async fn register_routes() -> Router {
    let router = Router::new()
        .route(&format!("/{}", PLANET_LIST), get(planetlist::get))
        .route(&format!("/{}", CHAT_ANNOUNCEMENT), get(chat_announcement::get))
        .route(&format!("/{}", GAMEFIND_MAIN), get(userlist::get))
        .route(&format!("/{}", GAMEBOARD_FREE_LIST), get(noop::get))
        .route(&format!("/{}", GAMEBOARD_JAMJJANG), get(noop::get))
        .route(&format!("/{}", GAMEBOARD_SCREEN_LIST), get(noop::get))
        .route(&format!("/{}", GAMEBOARD_NOTE_LIST), get(noop::get))
        .route(&format!("/{}", GAMEBOARD_NEMO_LIST), get(noop::get))
        .route(&format!("/{}", VIP_INDEX), get(noop::get))
        .route(&format!("/{}", GAMEMUSIC), get(noop::get))
        .route(&format!("/{}", MUSICCAPSULE), get(noop::get))
        .route(&format!("/{}", GIFT_INPUT), get(noop::get))
        .route(&format!("/{}", PAYMENT_INPUT), post(payment::post))
        .route("/api/users", get(getusers::get))
        .route("/api/login", post(login::post))
        .route("/api/registration", get(registration::get))
        .route("/api/payment", post(payment::post2))
        .route("/registration", post(registration::post))
        .route("/", get(home::get))
        .fallback(|req: axum::http::Request<axum::body::Body>| async move {
            let url = req.uri().to_string();

            if req.method() == axum::http::Method::POST {
                let (_parts, body) = req.into_parts();

                let body_str = match to_bytes(body, usize::MAX).await {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => "<Failed to read body>".to_string(),
                };

                log::info!("404 path: {}: {}", url, body_str);

                return (
                    axum::http::StatusCode::NOT_FOUND,
                    axum::Json(serde_json::json!({
                        "error": "Not Found"
                    })),
                ).into_response();
            } else {
                log::info!("404 path: {}", url);

                if url.starts_with("/api/") || url == "/api" {
                    return (
                        axum::http::StatusCode::NOT_FOUND,
                        axum::Json(serde_json::json!({
                            "error": "Not Found"
                        })),
                    ).into_response();
                }

                return (
                    axum::http::StatusCode::NOT_FOUND,
                    axum::response::Html("<h1>404 Not Found</h1>".to_string()),
                ).into_response();
            }
        });

    router
}

macro_rules! parse_query_params {
    ($req:expr) => {{
        use std::collections::HashMap;

        let (_parts, body) = $req.into_parts();

        let body_str = match to_bytes(body, usize::MAX).await {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => "<Failed to read body>".to_string(),
        };

        let query_params: HashMap<String, String> =
            url::form_urlencoded::parse(body_str.as_bytes())
                .into_owned()
                .collect();

        query_params
    }};
}

pub(super) use parse_query_params;

macro_rules! parse_request {
    ($req:expr, $body_type:ty) => {{
        use axum::body::to_bytes;
        use axum::response::IntoResponse as _;

        let (_parts, body) = $req.into_parts();

        let body_str = match to_bytes(body, usize::MAX).await {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    "Failed to read request body".to_string(),
                )
                    .into_response();
            }
        };

        match serde_json::from_str::<$body_type>(&body_str) {
            Ok(body) => body,
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    "Failed to parse request body".to_string(),
                )
                    .into_response();
            }
        }
    }};
}

pub(super) use parse_request;

macro_rules! make_response_json {
    ($status:expr, $body:expr) => {{
        use axum::{
            response::{IntoResponse, Json},
        };

        (
            $status,
            Json($body),
        )
            .into_response()
    }};
}


pub(super) use make_response_json;