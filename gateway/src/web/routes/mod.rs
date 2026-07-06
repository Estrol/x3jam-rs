use axum::{
    Router,
    body::to_bytes,
    routing::{get, post},
};

pub mod chat_announcement;
pub mod getusers;
pub mod payment;
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
        .route("/payment/payment_input.asp", post(payment::handle))
        .route("/api/payment", post(payment::payment))
        .fallback(axum::routing::get(
            |req: axum::http::Request<axum::body::Body>| async move {
                // log the requested path for unmatched routes
                println!("404 path: {}", req.uri());

                axum::response::Response::builder()
                    .status(404)
                    .body("Not Found".to_string())
                    .unwrap()
            },
        ))
        .fallback(axum::routing::post(
            |req: axum::http::Request<axum::body::Body>| async move {
                let (parts, body) = req.into_parts();

                let body_str = match to_bytes(body, usize::MAX).await {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => "<Failed to read body>".to_string(),
                };

                println!("404 path: {}: {}", parts.uri, body_str);

                axum::response::Response::builder()
                    .status(404)
                    .body("Not Found".to_string())
                    .unwrap()
            },
        ));

    router
}
