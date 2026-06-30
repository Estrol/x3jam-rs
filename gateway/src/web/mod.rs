pub mod routes;

pub async fn run(
    token: tokio_util::sync::CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting web server on port 16000...");

    let router = routes::register_routes().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:16000").await?;

    tokio::select! {
        _ = axum::serve(listener, router.into_make_service()) => {},
        _ = token.cancelled() => {
            println!("Web server is shutting down...");
        }
    }

    Ok(())
}
