pub async fn get() -> impl axum::response::IntoResponse {
    // Redirect to rickroll page, because this is a placeholder for the home page.

    axum::response::Redirect::temporary("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
}