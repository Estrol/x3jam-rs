pub async fn get() -> impl axum::response::IntoResponse {
    #[allow(unused_mut)] // TODO: add format options to announcement.txt and use them here
    let mut text = tokio::fs::read_to_string("./resources/web/announcement.txt")
        .await
        .unwrap_or_default();

    axum::response::Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(text)
        .unwrap()
}
