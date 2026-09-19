pub mod routes;

pub async fn run(
    token: tokio_util::sync::CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = crate::config::get::<u32>("CONFIG", "WebPort", 16000);
    let addr = format!("0.0.0.0:{}", port);
    log::info!("Starting web server on {}...", addr);

    let router = routes::register_routes().await;

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tokio::select! {
        _ = axum::serve(listener, router.into_make_service()) => {},
        _ = token.cancelled() => {
            log::info!("Web server is shutting down...");
        }
    }

    Ok(())
}
