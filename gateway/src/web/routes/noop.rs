pub async fn get() -> impl axum::response::IntoResponse {
    let file = tokio::fs::read("./resources/web/noop.html")
        .await
        .unwrap_or_default();

    axum::response::Html(file)
}