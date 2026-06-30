use axum::{Router, routing::get};

pub mod chat_announcement;
pub mod getusers;
pub mod planetlist;
pub mod userlist;

pub async fn register_routes() -> Router {
    let router = Router::new()
        .route("/Patch/a.html", get(planetlist::handle))
        .route("/Patch/b.txt", get(chat_announcement::handle))
        .route("/userlist.aspx", get(userlist::handle))
        .route("/gamefind/gamefine_main.asp", get(userlist::handle))
        .route("/gamefind/gamefine_main.aspx", get(userlist::handle))
        .route("/api/getusers", get(getusers::handle))
        .fallback(axum::routing::get(|| async {
            axum::response::Response::builder()
                .status(404)
                .body("Not Found".to_string())
                .unwrap()
        }));

    router
}
